//! Rendering to the terminal.
use std::io::Write;

use crossterm::{
    cursor,
    style::{self, Color, Print},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
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

/// Instruction to [Renderer] about what should be drawn to the screen.
#[derive(Debug)]
pub enum Draw {
    /// Draw the data, i.e. the text from which the selection is performed.
    Data,
    /// Draw the provided text at a location relative to the data.
    ///
    /// Being relative to data and not screen coordinates, allows the [Renderer]
    /// to draw the text over a certain word regardless of the screen size.
    ///
    /// For example, for the data `"this is a test` and location 10, the text will
    /// be drawn over the word `"test"` regardless of whether it is in the first
    /// terminal row or second (due to potential wrapping on a very small terminal).
    TextRelativeToData {
        /// The text to draw.
        text: String,
        /// Location relative to data at which to draw the text.
        location: usize,
        /// The style of the text to draw.
        style: TextStyle,
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
    pub fn render(&mut self, data: &str, draw_instructions: &[Draw]) {
        trace!("Rendering draw instructions {:#?}", draw_instructions);

        self.output.queue(Clear(ClearType::All)).unwrap();

        for instruction in draw_instructions {
            match instruction {
                Draw::Data => self.draw_data(data),
                Draw::TextRelativeToData {
                    text,
                    location,
                    style,
                } => self.draw_text_relative_to_data(text, data, *location, style),
            }
        }

        self.output.flush().unwrap();
    }

    /// Render the given data to the screen, taking into account new lines
    /// and terminal width overflow.
    fn draw_data(&mut self, data: &str) {
        // TODO This function assumes that each line will be smaller
        // or equal to screen width. Take into account that the line
        // can overflow.
        let (_, rows) = terminal::size().unwrap();
        data.lines()
            .enumerate()
            .take(rows as usize)
            .for_each(|(row, line)| {
                self.output.queue(cursor::MoveTo(0, row as u16)).unwrap();
                self.output.queue(Print(line)).unwrap();
            });
    }

    /// Render the given text at the given location.
    ///
    /// See documentation for [Draw::TextRelativeToData] for explanation
    /// on what "relative to data" means.
    fn draw_text_relative_to_data(
        &mut self,
        text: &str,
        data: &str,
        location: usize,
        style: &TextStyle,
    ) {
        let (cols, rows) = terminal::size().unwrap();
        let (row, col) = data_location_to_screen_location(data, rows, cols, location);

        self.output.queue(cursor::MoveTo(col, row)).unwrap();
        self.output
            .queue(style::SetForegroundColor(style.foreground))
            .unwrap();
        self.output
            .queue(style::SetBackgroundColor(style.background))
            .unwrap();
        self.output.queue(style::Print(text)).unwrap();
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

/// Convert the given location relative to data to a location
/// relative to screen.
///
/// Returns a tuple (row, col) representing the location.
fn data_location_to_screen_location(
    data: &str,
    _rows: u16,
    _cols: u16,
    location: usize,
) -> (u16, u16) {
    // TODO This function assumes that each line will be smaller
    // or equal to screen width. Take into account that the line
    // can overflow.
    let (row, row_start) = data[..=location].chars().enumerate().fold(
        (0, 0),
        |(row, row_start), (char_index, char)| {
            if char == '\n' {
                (row + 1, char_index + 1)
            } else {
                (row, row_start)
            }
        },
    );

    let col = location - row_start;
    (row, col as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("test", 10, 10, 0, (0, 0))]
    #[test_case("test", 10, 10, 1, (0, 1))]
    #[test_case("test\nmore test", 10, 10, 5, (1, 0))]
    #[test_case("test\nmore test", 10, 10, 10, (1, 5))]
    #[test_case("\n\ntest", 10, 10, 3, (2, 1))]
    fn data_location_to_screen_location_returns_expected_values(
        data: &str,
        rows: u16,
        cols: u16,
        location: usize,
        expected: (u16, u16),
    ) {
        let location = data_location_to_screen_location(data, rows, cols, location);

        assert_eq!(location, expected);
    }
}
