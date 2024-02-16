use std::{collections::HashMap, iter};

use crossterm::style::Color;
use regex::Regex;

use crate::{
    input_handler::KeyPress,
    renderer::{Draw, TextStyle},
};

use super::{Mode, ModeAction};

struct Hit {
    location: usize,
}

pub struct RegexMode {
    hint_hit_map: HashMap<String, Hit>,
}

// TODO Make sure that the configuration for RegexMode is
// read from config arguments

impl RegexMode {
    pub fn new(data: &str, regexes: &[String]) -> Self {
        // TODO Implement a more reasonable way of choosing hints
        let mut hint_pool = "asdfghjkl;qwertyuiopzxcvbnm".chars().cycle();
        let mut hint_hit_map = HashMap::new();

        for regex in regexes {
            let regex = Regex::new(regex).unwrap();
            let matches = regex.find_iter(data);

            for regex_match in matches {
                let hint = hint_pool.next().unwrap();

                hint_hit_map.insert(
                    hint.to_string(),
                    Hit {
                        location: regex_match.start(),
                    },
                );
            }
        }

        Self { hint_hit_map }
    }
}

impl Mode for RegexMode {
    fn handle_key_press(&mut self, _key: KeyPress) -> Option<ModeAction> {
        None
    }

    fn get_draw_instructions(&self) -> Vec<Draw> {
        let matches = self
            .hint_hit_map
            .iter()
            .map(|(hint, hit)| Draw::TextRelativeToData {
                text: hint.clone(),
                location: hit.location,
                style: TextStyle {
                    foreground: Color::Red,
                    background: Color::Yellow,
                },
            });

        iter::once(Draw::Data).chain(matches).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_instructions_at_expected_locations() {
        let text = "things and stuff";
        let regexes = [r"[a-z]{4,}".to_string()];

        let mode = RegexMode::new(text, &regexes);
        let hits: Vec<usize> = mode
            .get_draw_instructions()
            .into_iter()
            .filter_map(|instruction| match instruction {
                Draw::TextRelativeToData { location, .. } => Some(location),
                _ => None,
            })
            .collect();

        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&0)); // hit things
        assert!(hits.contains(&11)); // hit stuff
    }
}
