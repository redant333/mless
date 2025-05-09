//!Renderer struct that performs the actual rendering to the terminal.
use std::{collections::VecDeque, io::Write};

use crossterm::{
    cursor::{self, MoveTo},
    style::{self, Attribute, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
    QueueableCommand,
};
use log::trace;
use snafu::ResultExt;

use crate::error::RunError;
use crate::{configuration, error::IoSnafu};

use super::ansi_sequence_extractor::AnsiSequenceExtractor;
use super::{DataOverlay, StyledSegment, TextStyle};

use super::DrawInstruction;

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
    pub fn render(
        &mut self,
        data: &str,
        draw_instructions: &[DrawInstruction],
        config: &configuration::Config,
    ) -> Result<(), RunError> {
        trace!("Rendering draw instructions {:#?}", draw_instructions);

        // Perform rendering into a buffer first, to avoid any blinking issues
        let mut buffer: Vec<u8> = vec![];

        // Make sure the rendering starts from a predictable state every time
        buffer //
            .queue(ResetColor)
            .context(IoSnafu {})?
            .queue(SetAttribute(Attribute::Reset))
            .context(IoSnafu {})?
            .queue(Clear(ClearType::All))
            .context(IoSnafu {})?
            .queue(MoveTo(0, 0))
            .context(IoSnafu {})?
            .queue(EnableLineWrap)
            .context(IoSnafu {})?;

        for instruction in draw_instructions {
            match instruction {
                DrawInstruction::StyledData {
                    styled_segments,
                    text_overlays,
                } => {
                    self.draw_styled_data(&mut buffer, data, styled_segments, text_overlays)?;
                }
                DrawInstruction::Data => self.draw_styled_data(&mut buffer, data, &[], &[])?,
                DrawInstruction::ModeSelectionDialog(modes) => {
                    self.draw_mode_selection_dialog(&mut buffer, modes, config)?
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

    /// Draw the mode selection dialog. The styling of the dialog is completely controled
    /// by the renderer.
    fn draw_mode_selection_dialog(
        &mut self,
        buffer: &mut Vec<u8>,
        modes: &[(char, String)],
        config: &configuration::Config,
    ) -> Result<(), RunError> {
        let dialog_width: usize = config.mode_switch_width;
        let start_row = 1; // to have a top padding

        let (cols, rows) = terminal::size().context(IoSnafu {})?;

        // If there is not enough space to draw the dialog, just don't
        if cols <= dialog_width as u16 {
            return Ok(());
        }

        let mut modes_iter = modes.iter();
        // It's important to draw the spaces after the divider to make
        // sure that any text underneath is not visible.
        let empty_row = format!("{:dialog_width$}", "│");

        // To make sure that any excess is not going to the new line
        buffer.queue(DisableLineWrap).context(IoSnafu {})?;

        for row in 0..rows {
            let start_col = cols - dialog_width as u16;

            // Draw the divider and spaces on
            buffer
                .queue(MoveTo(start_col, row))
                .context(IoSnafu {})?
                .queue(SetForegroundColor(config.mode_switch_divider_fg))
                .context(IoSnafu {})?
                .queue(Print(&empty_row))
                .context(IoSnafu {})?;

            if row >= start_row {
                if let Some((hotkey, name)) = modes_iter.next() {
                    buffer
                        .queue(MoveTo(start_col + 1, row))
                        .context(IoSnafu {})?
                        .queue(SetForegroundColor(config.mode_switch_hotkey_fg))
                        .context(IoSnafu {})?
                        .queue(Print(format!(" [{hotkey}] ")))
                        .context(IoSnafu {})?
                        .queue(ResetColor)
                        .context(IoSnafu {})?
                        .queue(SetForegroundColor(config.mode_switch_mode_name_fg))
                        .context(IoSnafu {})?
                        .queue(Print(&name))
                        .context(IoSnafu {})?;
                }
            }
        }

        buffer.queue(EnableLineWrap).context(IoSnafu {})?;
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
