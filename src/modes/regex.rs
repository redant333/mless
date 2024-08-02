//! A mode that allows selection based on a list of regexes.
//!
//! The idea behind this mode is to allow the user to provide a list
//! of regexes, and then select part of the text that matches any of them.
use std::collections::HashMap;

use crossterm::style::Color;
use log::trace;
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

        // TODO This will assign different hints to multiple appearences of the same
        // match text. Instead, every occurrence of the same match should get the
        // same hint since it will give the same output.

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

        let hints = hint_generator.create_hints(hits.len());

        let hint_hit_map = std::iter::zip(hints, hits).collect();

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

        if let Some(hit) = self.hint_hit_map.get(&self.input_buffer) {
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
            .values()
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

#[cfg(test)]
mod tests {
    use crate::{configuration::RegexArgs, hints::MockHintGenerator};
    use test_case::test_case;

    use super::*;

    fn has_overlay_at_location(overlays: &[DataOverlay], location: usize) -> bool {
        overlays.iter().any(|overlay| overlay.location == location)
    }

    fn has_highlight(highlights: &[StyledSegment], start: usize, length: usize) -> bool {
        highlights
            .iter()
            .any(|highlight| highlight.start == start && highlight.length == length)
    }

    fn get_draw_instructions(
        text: &str,
        regexes: Vec<String>,
        hints: Vec<String>,
    ) -> (Vec<DataOverlay>, Vec<StyledSegment>) {
        let args = RegexArgs { regexes };

        let mut hint_generator = Box::new(MockHintGenerator::new());
        hint_generator.expect_create_hints().return_const(hints);

        let mode = RegexMode::new(text, &args, hint_generator);
        let Draw::StyledData {
            text_overlays,
            styled_segments,
        } = mode.get_draw_instructions().into_iter().next().unwrap();

        (text_overlays, styled_segments)
    }

    #[test]
    fn produces_expected_highlights_and_overlays_for_simple_text() {
        let (text_overlays, styled_segments) = get_draw_instructions(
            "things and stuff",
            vec![r"[a-z]{4,}".into()],
            vec!["a".into(), "b".into()],
        );

        assert_eq!(text_overlays.len(), 2);
        assert!(has_overlay_at_location(&text_overlays, 0));
        assert!(has_overlay_at_location(&text_overlays, 11));

        assert_eq!(styled_segments.len(), 4);
        // Highlights for "things" match
        assert!(has_highlight(&styled_segments, 0, 6));
        assert!(has_highlight(&styled_segments, 0, 1));

        // Highlights for "stuff" match
        assert!(has_highlight(&styled_segments, 11, 5));
        assert!(has_highlight(&styled_segments, 11, 1));
    }

    #[test]
    fn produces_expected_highlights_and_overlays_for_colored_text() {
        let color_1 = "\x1b[0;31m";
        let color_2 = "\x1b[0;32m";
        let color_reset = "\x1b[0m";

        let (text_overlays, styled_segments) = get_draw_instructions(
            &format!("{color_1}things{color_reset} and {color_2}stuff{color_reset} and {color_1}mice{color_reset} and {color_2}mud{color_reset}"),
            vec![r"[a-z]{4,}".into()],
            vec!["a".into(), "b".into(), "c".into()],
        );

        println!("{:#?}", text_overlays);
        println!("{:#?}", styled_segments);

        assert_eq!(text_overlays.len(), 3);
        assert!(has_overlay_at_location(&text_overlays, 7));
        assert!(has_overlay_at_location(&text_overlays, 29));
        assert!(has_overlay_at_location(&text_overlays, 50));

        assert_eq!(styled_segments.len(), 6);
        // Highlights for "things" match
        assert!(has_highlight(&styled_segments, 7, 6));
        assert!(has_highlight(&styled_segments, 7, 1));

        // Highlights for "stuff" match
        assert!(has_highlight(&styled_segments, 29, 5));
        assert!(has_highlight(&styled_segments, 29, 1));

        // Highlights for "mice" match
        assert!(has_highlight(&styled_segments, 50, 4));
        assert!(has_highlight(&styled_segments, 50, 1));
    }

    #[test_case(&[(2,4), (6, 8)], 0, 0)]
    #[test_case(&[(2,4), (6, 8)], 1, 1)]
    #[test_case(&[(2,4), (6, 8)], 2, 4)]
    #[test_case(&[(2,4), (6, 8)], 3, 5)]
    #[test_case(&[(2,4), (6, 8)], 4, 8)]
    #[test_case(&[], 4, 4)]
    fn get_original_index_returns_correct_value(
        removed_ranges: &[(usize, usize)],
        index: usize,
        expected: usize,
    ) {
        assert_eq!(get_original_index(removed_ranges, index), expected);
    }
}
