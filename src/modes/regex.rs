//! A mode that allows selection based on a list of regexes.
//!
//! The idea behind this mode is to allow the user to provide a list
//! of regexes, and then select part of the text that matches any of them.
use std::{collections::HashMap, iter};

use crossterm::style::Color;
use regex::Regex;

use crate::{
    configuration,
    hints::HintGenerator,
    input_handler::KeyPress,
    renderer::{Draw, TextStyle},
};

use super::{Mode, ModeEvent};

/// Struct that records a hit(match) that can be selected.
struct Hit {
    /// Location of the hit within the data.
    ///
    /// This is represented as character offset from the first character.
    location: usize,

    /// The text of the hit.
    ///
    /// This will be returned to the user if this hit is selected.
    text: String,
}

/// Struct representing the regex selection mode.
pub struct RegexMode {
    /// A map between the hint (sequence of characters that select a hit) and
    /// the [Hit] struct itself containing the details of the hit.
    hint_hit_map: HashMap<String, Hit>,

    /// The sequence of characters pressed so far.
    ///
    /// This is needed for situations when selecting any hit requires at least
    /// two key presses.
    input_buffer: String,
}

impl RegexMode {
    /// Create a new regex mode for selecting from the given data with the given args.
    pub fn new(
        data: &str,
        args: &configuration::RegexArgs,
        hint_generator: Box<dyn HintGenerator>,
    ) -> Self {
        let mut hits = vec![];

        // TODO This will assign the same hint to multiple appearences of the same
        // match text. Instead, every occurrence of the same word should get the
        // same hint since it will give the same output.

        for regex in &args.regexes {
            let regex = Regex::new(regex).unwrap();
            regex
                .find_iter(data)
                .map(|regex_match| Hit {
                    location: regex_match.start(),
                    text: regex_match.as_str().to_string(),
                })
                .for_each(|hit| hits.push(hit));
        }

        let hints = hint_generator.create_hints(hits.len());

        let hint_hit_map = std::iter::zip(hints, hits).collect();

        Self {
            hint_hit_map,
            input_buffer: String::new(),
        }
    }
}

impl Mode for RegexMode {
    fn handle_key_press(&mut self, key: KeyPress) -> Option<ModeEvent> {
        self.input_buffer.push(key.key);

        if let Some(hit) = self.hint_hit_map.get(&self.input_buffer) {
            self.input_buffer.clear();
            Some(ModeEvent::TextSelected(hit.text.clone()))
        } else {
            None
        }
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
    use crate::{configuration::RegexArgs, hints::MockHintGenerator};

    use super::*;

    #[test]
    fn produces_instructions_at_expected_locations() {
        let text = "things and stuff";
        let args = RegexArgs {
            regexes: vec![r"[a-z]{4,}".to_string()],
        };

        let mut hint_generator = Box::new(MockHintGenerator::new());
        hint_generator
            .expect_create_hints()
            .return_const(vec!["a".to_string(), "b".to_string()]);

        let mode = RegexMode::new(text, &args, hint_generator);
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
