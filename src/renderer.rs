//! Rendering to the terminal.
use std::{collections::VecDeque, io::Write};

use crossterm::{
    cursor,
    style::{self, Color, Print},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};
use log::trace;

/// Struct to describe text style.
///
/// Used in [Draw].
#[derive(Debug)]
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

// TODO Currently, if data is longer than the screen, it will be drawn until the end and cause
// scrolling in the terminal. This needs to be fixed so that only the first screen is drawn and
// the rest is ignored.

impl<T: Write + ?Sized> Renderer<T> {
    /// Render the given data and draw instructions to the terminal.
    ///
    /// Draw instructions are executed in the given order.
    pub fn render(&mut self, data: &str, draw_instructions: &[Draw]) {
        trace!("Rendering draw instructions {:#?}", draw_instructions);

        self.output.queue(Clear(ClearType::All)).unwrap();

        for instruction in draw_instructions {
            match instruction {
                Draw::StyledData {
                    styled_segments,
                    text_overlays,
                } => {
                    self.draw_styled_data(data, styled_segments, text_overlays);
                }
            }
        }

        self.output.flush().unwrap();
    }

    /// Render styled parts of data to the screen, taking into account new lines
    /// and terminal width overflow.
    fn draw_styled_data(
        &mut self,
        data: &str,
        styled_segments: &[StyledSegment],
        text_overlays: &[DataOverlay],
    ) {
        let mut overlay_chars: VecDeque<char> = VecDeque::new();
        let mut color_stack: Vec<(Color, Color)> = vec![];
        let mut refresh_colors = false;

        // TODO Make sure that styled segments do not inherit text properties like bold
        // from the outer text

        // Ignore the terminating new line if present
        let data_range = match data.as_bytes().last() {
            Some(b'\n') => 0..(data.len() - 1),
            _ => 0..data.len(),
        };

        for (byte_position, char) in data[data_range].char_indices() {
            // Handle end of segment
            styled_segments
                .iter()
                .filter(|s| (s.start + s.length) == byte_position)
                .for_each(|_| {
                    color_stack.pop();
                    refresh_colors = true;
                });

            // Handle start of segment
            styled_segments
                .iter()
                .filter(|s| s.start == byte_position)
                .for_each(|segment| {
                    color_stack.push((segment.style.background, segment.style.foreground));
                    refresh_colors = true;
                });

            // Handle start of overlay
            let overlay = text_overlays
                .iter()
                .find(|overlay| overlay.location == byte_position);

            if let Some(DataOverlay { text, .. }) = overlay {
                text.chars().for_each(|char| overlay_chars.push_back(char));
            }

            // Change color if needed
            if refresh_colors {
                if let Some((background, foreground)) = color_stack.last() {
                    self.output
                        .queue(style::SetForegroundColor(*foreground))
                        .unwrap();
                    self.output
                        .queue(style::SetBackgroundColor(*background))
                        .unwrap();
                } else {
                    self.output.queue(style::ResetColor).unwrap();
                }

                refresh_colors = false;
            }

            // Print character
            let char = match overlay_chars.pop_front() {
                Some(overlay_char) => overlay_char,
                None => char,
            };

            if char == '\n' {
                self.output.queue(Print('\r')).unwrap();
            }
            self.output.queue(Print(char)).unwrap();
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
