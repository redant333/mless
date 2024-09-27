//!Renderer struct that performs the actual rendering to the terminal.
use std::{collections::VecDeque, io::Write};

use crossterm::{
    cursor,
    style::{self, Print},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};
use log::trace;
use snafu::ResultExt;

use crate::{IoSnafu, RunError};

use super::ansi_sequence_extractor::AnsiSequenceExtractor;
use super::{DataOverlay, StyledSegment, TextStyle};

use super::Draw;

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
    pub fn render(&mut self, data: &str, draw_instructions: &[Draw]) -> Result<(), RunError> {
        trace!("Rendering draw instructions {:#?}", draw_instructions);

        // Perform rendering into a buffer first, to avoid any blinking issues
        let mut buffer: Vec<u8> = vec![];
        self.output
            .queue(Clear(ClearType::All))
            .context(IoSnafu {})?;

        for instruction in draw_instructions {
            match instruction {
                Draw::StyledData {
                    styled_segments,
                    text_overlays,
                } => {
                    self.draw_styled_data(&mut buffer, data, styled_segments, text_overlays)?;
                }
            }
        }

        self.output.write_all(&buffer).context(IoSnafu {})?;
        self.output.flush().context(IoSnafu {})?;

        Ok(())
    }

    /// Render styled parts of data to the screen, taking into account new lines
    /// and terminal width overflow.
    fn draw_styled_data(
        &mut self,
        buffer: &mut Vec<u8>,
        data: &str,
        styled_segments: &[StyledSegment],
        text_overlays: &[DataOverlay],
    ) -> Result<(), RunError> {
        let mut overlay_chars: VecDeque<char> = VecDeque::new();
        let ansi_sequences = AnsiSequenceExtractor::new(data)?;
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

            self.update_style(
                &last_intra_segment_style,
                &intra_segment_style,
                &ansi_sequences,
                buffer,
                byte_position,
            )?;
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
                    buffer.queue(Print('\r')).context(IoSnafu {})?;
                }
                buffer.queue(Print(char)).context(IoSnafu {})?;
            }
        }
        Ok(())
    }

    /// Update the terminal style when switching in and out of styled segments
    fn update_style(
        &self,
        last_segment_style: &Option<TextStyle>,
        segment_style: &Option<TextStyle>,
        ansi_sequences: &AnsiSequenceExtractor,
        buffer: &mut Vec<u8>,
        current_position: usize,
    ) -> Result<(), RunError> {
        use style::*;

        match (last_segment_style, segment_style) {
            (Some(_), None) => {
                // Just exited from a styled segment, restore any styling disturbed by it
                buffer
                    .queue(SetAttribute(Attribute::Reset))
                    .context(IoSnafu {})?;
                buffer.queue(ResetColor).context(IoSnafu {})?;

                // In order to restore the styling, this applies all the sequences
                // from the beginning of the data again. This is a fairly silly approach
                // but it means that the code does not need to worry about which styles
                // overried which and similar.
                for sequence in ansi_sequences.get_all_sequences_before(current_position) {
                    buffer.queue(Print(sequence)).context(IoSnafu {})?;
                }
            }
            (None, Some(style)) => {
                // Just entered a segment, apply its style
                buffer
                    .queue(SetAttribute(Attribute::Reset))
                    .context(IoSnafu {})?;
                buffer
                    .queue(SetForegroundColor(style.foreground))
                    .context(IoSnafu {})?;
                buffer
                    .queue(SetBackgroundColor(style.background))
                    .context(IoSnafu {})?;
            }
            (Some(last_style), Some(style)) if last_style != style => {
                // Just switched from one segment to another, apply the style
                // of the new segment
                buffer
                    .queue(SetAttribute(Attribute::Reset))
                    .context(IoSnafu {})?;
                buffer
                    .queue(SetForegroundColor(style.foreground))
                    .context(IoSnafu {})?;
                buffer
                    .queue(SetBackgroundColor(style.background))
                    .context(IoSnafu {})?;
            }
            _ => (),
        }
        Ok(())
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
