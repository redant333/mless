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

pub struct TextStyle {
    pub foreground: Color,
    pub background: Color,
}

pub enum Draw {
    Data,
    TextRelativeToData {
        text: String,
        location: usize,
        style: TextStyle,
    },
}

pub struct Renderer<T: Write + ?Sized> {
    pub output: T,
}

impl<T: Write + ?Sized> Renderer<T> {
    pub fn render(&mut self, data: &str, draw_instructions: &[Draw]) {
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

    pub fn initialize_terminal(&mut self) -> std::io::Result<()> {
        self.output
            .queue(cursor::Hide)?
            .queue(EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    pub fn uninitialize_terminal(&mut self) -> std::io::Result<()> {
        self.output
            .queue(cursor::Show)?
            .queue(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }
}

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
