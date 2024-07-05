//! A mode that allows selection based on a list of regexes.
//!
//! The idea behind this mode is to allow the user to provide a list
//! of regexes, and then select part of the text that matches any of them.
use std::collections::HashMap;

use crossterm::style::Color;
use regex::Regex;

use crate::{
    configuration,
    hints::HintGenerator,
    input_handler::KeyPress,
    renderer::{DataOverlay, Draw, StyledSegment, TextStyle},
};

use super::{Mode, ModeEvent};

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

        // If there is an ANSI color sequence before the match, it should be ignored.
        // For that reason, add the ignore regex to the beginning and the end of the
        // regex being searched with a ? operator. If it's there, it won't be included
        // in the main match, if it isn't, it will just be ignored.
        // This can be extended for anything else that needs to be ignored.
        let ignore_regex = "\x1b\\[[^m]+m".to_string();

        for regex in &args.regexes {
            let wrapped_regex = format!("({ignore_regex})?({regex})({ignore_regex})?");
            let regex = Regex::new(&wrapped_regex).unwrap();

            regex
                .captures_iter(data)
                .filter_map(|capture| {
                    let regex_match = capture.get(2)?;

                    Some(Hit {
                        start: regex_match.start(),
                        length: regex_match.as_str().len(),
                        text: regex_match.as_str().to_string(),
                    })
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

#[cfg(test)]
mod tests {
    use crate::{configuration::RegexArgs, hints::MockHintGenerator};

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
        let (text_overlays, styled_segments) = get_draw_instructions(
            "\x1b[94mthings\x1b[0m and \x1b[93mstuff\x1b[0m",
            vec![r"[a-z]{4,}".into()],
            vec!["a".into(), "b".into()],
        );

        assert_eq!(text_overlays.len(), 2);
        assert!(has_overlay_at_location(&text_overlays, 5));
        assert!(has_overlay_at_location(&text_overlays, 25));

        assert_eq!(styled_segments.len(), 4);
        // Highlights for "things" match
        assert!(has_highlight(&styled_segments, 5, 6));
        assert!(has_highlight(&styled_segments, 5, 1));

        // Highlights for "stuff" match
        assert!(has_highlight(&styled_segments, 25, 5));
        assert!(has_highlight(&styled_segments, 25, 1));
    }
}
