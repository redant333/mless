//! A mode that allows selection based on a list of regexes.
//!
//! The idea behind this mode is to allow the user to provide a list
//! of regexes, and then select part of the text that matches any of them.
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crossterm::style::Color;
use log::{info, trace};
use regex::Regex;

use crate::{
    configuration,
    hints::HintGenerator,
    input_handler::KeyPress,
    renderer::{DataOverlay, Draw, StyledSegment, TextStyle},
};

use super::{Mode, ModeEvent};

#[derive(Debug)]
/// Struct that records a hit(match) that can be selected.
struct Hit {
    /// Byte offset of the start of the hit.
    ///
    /// This is represented as character offset from the first character.
    start: usize,

    /// Length of the hit in bytes.
    length: usize,

    /// The text of the hit.
    ///
    /// This will be returned to the user if this hit is selected.
    text: String,
}

/// Struct representing the regex selection mode.
pub struct RegexMode {
    /// A map between the hint (sequence of characters that select a hit) and
    /// the [Hit] struct itself containing the details of the hit.
    hint_hit_map: Vec<(String, Hit)>,

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

        // All ANSI color sequences should be ignored while matching
        let ignore_regex = Regex::new("\x1b\\[[^m]+m").unwrap();

        let ignore_ranges = ignore_regex
            .captures_iter(data)
            .map(|captures| {
                let regex_match = captures.get(0).unwrap();
                (regex_match.start(), regex_match.end())
            })
            .collect::<Vec<(usize, usize)>>();

        // Remove all the ignored sequences and perform the matching
        // on the resulting data
        let cleaned_data = ignore_regex.replace_all(data, "");

        for regex in &args.regexes {
            let regex = Regex::new(regex).unwrap();

            regex
                .captures_iter(&cleaned_data)
                .filter_map(|capture| {
                    let regex_match = capture.get(0)?;

                    let start_before_ignored =
                        get_original_index(&ignore_ranges, regex_match.start());
                    Some(Hit {
                        start: start_before_ignored,
                        length: regex_match.as_str().len(),
                        text: regex_match.as_str().to_string(),
                    })
                })
                .for_each(|hit| hits.push(hit));
        }

        let hint_hit_map = into_hint_hit_map(hits, hint_generator.deref());

        trace!("Constructed hint hit map {:#?}", hint_hit_map);

        Self {
            hint_hit_map,
            input_buffer: String::new(),
        }
    }
}

impl Mode for RegexMode {
    fn handle_key_press(&mut self, key: KeyPress) -> Option<ModeEvent> {
        self.input_buffer.push(key.key);

        let matching_hit = self.hint_hit_map.iter().find_map(|(hint, hit)| {
            if *hint == self.input_buffer {
                Some(hit)
            } else {
                None
            }
        });

        if let Some(hit) = matching_hit {
            self.input_buffer.clear();
            Some(ModeEvent::TextSelected(hit.text.clone()))
        } else {
            None
        }
    }

    fn get_draw_instructions(&self) -> Vec<Draw> {
        let hint_fg = Color::parse_ansi("5;232").unwrap();
        let hint_bg = Color::parse_ansi("5;208").unwrap();

        let highlight_fg = Color::parse_ansi("5;232").unwrap();
        let highlight_bg = Color::parse_ansi("5;252").unwrap();

        let mut highlights: Vec<StyledSegment> = self
            .hint_hit_map
            .iter()
            .map(|(_, hit)| hit)
            .map(|hit| StyledSegment {
                start: hit.start,
                length: hit.length,
                style: TextStyle {
                    foreground: highlight_fg,
                    background: highlight_bg,
                },
            })
            .collect();

        let (hint_highlights, overlays): (Vec<StyledSegment>, Vec<DataOverlay>) = self
            .hint_hit_map
            .iter()
            .map(|(hint, hit)| {
                let highlight = StyledSegment {
                    start: hit.start,
                    length: hint.len(),
                    style: TextStyle {
                        foreground: hint_fg,
                        background: hint_bg,
                    },
                };

                let overlay = DataOverlay {
                    location: hit.start,
                    text: hint.clone(),
                };

                (highlight, overlay)
            })
            .unzip();

        highlights.extend(hint_highlights);

        vec![Draw::StyledData {
            styled_segments: highlights,
            text_overlays: overlays,
        }]
    }
}

/// For a sequence from which `removed_ranges` where removed, find the index that
/// the element had before removal for the given `index_after_removal`.
///
/// `removed_ranges` represents the collection of ranges (a, b) where a is included
/// in the range and b is not.
///
/// ```
/// // before removal [0, 1, 2, 3, 4, 5, 6]
/// // after removal  [0, 2, 3, 6]
/// let removed_ranges = [(1,2), (4,6)];
/// let index_after_removal = 3;
///
/// assert_eq!(get_original_index(removed_ranges, index_after_removal), 6);
/// ```
fn get_original_index(removed_ranges: &[(usize, usize)], index_after_removal: usize) -> usize {
    let mut offset_due_to_removed = 0;

    for &(start, end) in removed_ranges {
        if index_after_removal + offset_due_to_removed < start {
            return index_after_removal + offset_due_to_removed;
        }

        offset_due_to_removed += end - start;
    }

    index_after_removal + offset_due_to_removed
}

/// Create a mapping of hints to hits from the given collection of hits and the generator.
fn into_hint_hit_map(hits: Vec<Hit>, hint_generator: &dyn HintGenerator) -> Vec<(String, Hit)> {
    let unique_hit_count = hits
        .iter()
        .map(|hit| hit.text.clone())
        .collect::<HashSet<String>>()
        .len();
    info!("Number of unique hits {unique_hit_count}");
    let hints = hint_generator.create_hints(unique_hit_count);
    let mut hint_iter = hints.iter();

    let mut hit_hint_map = HashMap::<String, String>::new();
    let mut hint_hit_map: Vec<(String, Hit)> = vec![];

    for hit in hits.into_iter() {
        let hint = if hit_hint_map.contains_key(&hit.text) {
            trace!("Hit <{}> already in hit_hint_map", hit.text);
            hit_hint_map[&hit.text].clone()
        } else if let Some(hint) = hint_iter.next() {
            trace!("Using new hint {} for hit <{}>", hint, hit.text);
            hit_hint_map.insert(hit.text.clone(), hint.clone());
            hint.clone()
        } else {
            info!("Not enough hints for all the hits, giving up");
            break;
        };

        hint_hit_map.push((hint, hit));
    }

    hint_hit_map
}

#[cfg(test)]
mod tests {
    use super::*;
    mod tests_regex;
}
