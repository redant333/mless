//! Rendering to the terminal.
use std::{collections::VecDeque, io::Write, ops::Range};

use crossterm::{
    cursor,
    style::{self, Color, Print},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};
use log::{info, trace};
use regex::Regex;

/// Struct to describe text style.
///
/// Used in [Draw].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TextStyle {
    pub foreground: Color,
    pub background: Color,
}

/// Struct to describe a styled segment of data.
///
/// Used in [Draw::StyledData].
#[derive(Debug)]
pub struct StyledSegment {
    /// Byte offset of the start of the segment from the start of data.
    pub start: usize,
    /// Length of the segment in bytes.
    pub length: usize,
    /// Style of the segment.
    pub style: TextStyle,
}
/// Struct to describe text that is drawn over the data.
///
/// Used in [Draw::StyledData].
#[derive(Debug)]
pub struct DataOverlay {
    // The text to draw.
    pub text: String,
    /// Byte offset from the start of data where to start drawing the text.
    pub location: usize,
}

/// Instruction to [Renderer] about what should be drawn to the screen.
#[derive(Debug)]
pub enum Draw {
    /// Draw the data, i.e. the text from which the selection is performed
    /// with parts of data in different styles and text drawn over some parts.
    ///
    /// If some of the segments are overlapping, the last one specified takes precedence.
    StyledData {
        /// The segments of the data to style differently than the source data.
        styled_segments: Vec<StyledSegment>,
        /// Parts of the data to replace with the given text.
        text_overlays: Vec<DataOverlay>,
    },
}

/// The struct intended for rendering everything to the terminal.
///
/// Everything rendered to the terminal should come through the [Renderer::render] method.
pub struct Renderer<T: Write + ?Sized> {
    /// The output which the rendering is performed.
    ///
    /// The type of this field will likely be replaced with [std::io::Stdout] in the future.
    pub output: T,
}

impl<T: Write + ?Sized> Renderer<T> {
    /// Render the given data and draw instructions to the terminal.
    ///
    /// Draw instructions are executed in the given order.
    pub fn render(&mut self, data: &str, draw_instructions: &[Draw]) {
        trace!("Rendering draw instructions {:#?}", draw_instructions);

        // Perform rendering into a buffer first, to avoid any blinking issues
        let mut buffer: Vec<u8> = vec![];
        self.output.queue(Clear(ClearType::All)).unwrap();

        for instruction in draw_instructions {
            match instruction {
                Draw::StyledData {
                    styled_segments,
                    text_overlays,
                } => {
                    self.draw_styled_data(&mut buffer, data, styled_segments, text_overlays);
                }
            }
        }

        self.output.write_all(&buffer).unwrap();
        self.output.flush().unwrap();
    }

    /// Render styled parts of data to the screen, taking into account new lines
    /// and terminal width overflow.
    fn draw_styled_data(
        &mut self,
        buffer: &mut Vec<u8>,
        data: &str,
        styled_segments: &[StyledSegment],
        text_overlays: &[DataOverlay],
    ) {
        let mut overlay_chars: VecDeque<char> = VecDeque::new();
        let ansi_sequences = AnsiSequenceExtractor::new(data);
        let mut last_intra_segment_style = None;

        // Ignore the terminating new line if present
        let data_range = match data.as_bytes().last() {
            Some(b'\n') => 0..(data.len() - 1),
            _ => 0..data.len(),
        };

        for segment in styled_segments {
            trace!("Styled segment to draw {segment:?}")
        }

        for (byte_position, char) in data[data_range].char_indices() {
            // Handle start of overlay
            let overlay = text_overlays
                .iter()
                .find(|overlay| overlay.location == byte_position);

            if let Some(DataOverlay { text, .. }) = overlay {
                text.chars().for_each(|char| overlay_chars.push_back(char));
            }

            // Style from segments
            let intra_segment_style = styled_segments.iter().rev().find_map(|segment| {
                if byte_position >= segment.start
                    && byte_position < (segment.start + segment.length)
                {
                    Some(segment.style)
                } else {
                    None
                }
            });

            update_style(
                &last_intra_segment_style,
                &intra_segment_style,
                &ansi_sequences,
                buffer,
                byte_position,
            );
            last_intra_segment_style = intra_segment_style;

            // Do not print ANSI sequences inside of styled segments
            let inside_styled_segment = intra_segment_style.is_some();
            let current_char_is_ansi_sequence = ansi_sequences.is_inside_sequence(byte_position);

            if !(inside_styled_segment && current_char_is_ansi_sequence) {
                // Print character
                let char = match overlay_chars.pop_front() {
                    Some(overlay_char) => overlay_char,
                    None => char,
                };

                if char == '\n' {
                    buffer.queue(Print('\r')).unwrap();
                }
                buffer.queue(Print(char)).unwrap();
            }
        }

        // Update the terminal style when switching in and out of styled segments
        fn update_style(
            last_segment_style: &Option<TextStyle>,
            segment_style: &Option<TextStyle>,
            ansi_sequences: &AnsiSequenceExtractor,
            buffer: &mut Vec<u8>,
            current_position: usize,
        ) {
            use style::*;

            match (last_segment_style, segment_style) {
                (Some(_), None) => {
                    // Just exited from a styled segment, restore any styling disturbed by it
                    buffer.queue(SetAttribute(Attribute::Reset)).unwrap();
                    buffer.queue(ResetColor).unwrap();

                    // In order to restore the styling, this applies all the sequences
                    // from the beginning of the data again. This is a fairly silly approach
                    // but it means that the code does not need to worry about which styles
                    // overried which and similar.
                    for sequence in ansi_sequences.get_all_sequences_before(current_position) {
                        buffer.queue(Print(sequence)).unwrap();
                    }
                }
                (None, Some(style)) => {
                    // Just entered a segment, apply its style
                    buffer.queue(SetAttribute(Attribute::Reset)).unwrap();
                    buffer.queue(SetForegroundColor(style.foreground)).unwrap();
                    buffer.queue(SetBackgroundColor(style.background)).unwrap();
                }
                (Some(last_style), Some(style)) if last_style != style => {
                    // Just switched from one segment to another, apply the style
                    // of the new segment
                    buffer.queue(SetAttribute(Attribute::Reset)).unwrap();
                    buffer.queue(SetForegroundColor(style.foreground)).unwrap();
                    buffer.queue(SetBackgroundColor(style.background)).unwrap();
                }
                _ => (),
            }
        }
    }

    /// Prepare the terminal for the use by the application.
    pub fn initialize_terminal(&mut self) -> std::io::Result<()> {
        self.output
            .queue(cursor::Hide)?
            .queue(EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    /// Return the terminal to the initial state.
    ///
    /// Note that failing to run this function will almost certainly leave
    /// the terminal in an invalid, unusable state.
    pub fn uninitialize_terminal(&mut self) -> std::io::Result<()> {
        self.output
            .queue(cursor::Show)?
            .queue(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }
}

struct AnsiSequenceEntry {
    range: Range<usize>,
    content: String,
}

// A struct to extract and store all ANSI sequences in a string
struct AnsiSequenceExtractor {
    ansi_sequences: Vec<AnsiSequenceEntry>,
}

impl AnsiSequenceExtractor {
    // Create a new extractor from the given string
    fn new(data: &str) -> Self {
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

    // Check if the given byte location is inside any of the extracted
    // ANSI sequences
    fn is_inside_sequence(&self, location: usize) -> bool {
        self.ansi_sequences
            .iter()
            .any(|sequence| sequence.range.contains(&location))
    }

    // Get an iterator of all extracted ANSI sequences that end before (not including) the
    // given byte location
    fn get_all_sequences_before(&self, location: usize) -> Box<dyn Iterator<Item = &str> + '_> {
        let sequences = self
            .ansi_sequences
            .iter()
            .take_while(move |sequence| sequence.range.end < location)
            .map(|sequence| sequence.content.as_str());

        Box::new(sequences)
    }
}
