//! Struct for extraction and querying of ANSI sequences

use std::ops::Range;

use log::info;
use regex::Regex;

/// A struct to extract and store all ANSI sequences in a string
pub struct AnsiSequenceExtractor {
    ansi_sequences: Vec<AnsiSequenceEntry>,
}

/// One extracted ANSI sequence
struct AnsiSequenceEntry {
    range: Range<usize>,
    content: String,
}

impl AnsiSequenceExtractor {
    /// Create a new extractor from the given string
    pub fn new(data: &str) -> Self {
        let ansi_regex = Regex::new("\x1b\\[[^m]+m").unwrap();
        let ansi_sequences = ansi_regex
            .captures_iter(data)
            .map(|captures| {
                let regex_match = captures.get(0).unwrap();

                info!(
                    "Found ANSI sequence ({}, {})",
                    regex_match.start(),
                    regex_match.end()
                );
                AnsiSequenceEntry {
                    range: regex_match.start()..regex_match.end(),
                    content: regex_match.as_str().to_string(),
                }
            })
            .collect();

        Self { ansi_sequences }
    }

    /// Check if the given byte location is inside any of the extracted
    /// ANSI sequences
    pub fn is_inside_sequence(&self, location: usize) -> bool {
        self.ansi_sequences
            .iter()
            .any(|sequence| sequence.range.contains(&location))
    }

    /// Get an iterator of all extracted ANSI sequences that end before (not including) the
    /// given byte location
    pub fn get_all_sequences_before(&self, location: usize) -> Box<dyn Iterator<Item = &str> + '_> {
        let sequences = self
            .ansi_sequences
            .iter()
            .take_while(move |sequence| sequence.range.end < location)
            .map(|sequence| sequence.content.as_str());

        Box::new(sequences)
    }
}
