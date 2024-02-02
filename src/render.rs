use std::io::Write;

use crossterm::{
    cursor,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};

pub struct Renderer<T: Write + ?Sized> {
    pub rows: u16,
    pub cols: u16,
    pub output: T,
}

impl<T: Write + ?Sized> Renderer<T> {
    pub fn render(&mut self, data: &str) {
        self.output.queue(Clear(ClearType::All)).unwrap();

        data.lines()
            .enumerate()
            .take(self.rows as usize)
            .for_each(|(row, line)| {
                self.output.queue(cursor::MoveTo(0, row as u16)).unwrap();
                self.output.queue(Print(line)).unwrap();
            });

        self.output.flush().unwrap();
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
