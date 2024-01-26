use std::io::Write;

use crossterm::{
    cursor,
    style::Print,
    terminal::{Clear, ClearType},
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
}
