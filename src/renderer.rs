use std::io::Write;

use crossterm::{
    cursor,
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};

pub enum Draw {
    Data,
}

pub struct Renderer<T: Write + ?Sized> {
    pub output: T,
}

impl<T: Write + ?Sized> Renderer<T> {
    pub fn render(&mut self, data: &str, draw_instructions: &[Draw]) {
        self.output.queue(Clear(ClearType::All)).unwrap();

        for instruction in draw_instructions {
            match instruction {
                Draw::Data => {
                    let (_, rows) = terminal::size().unwrap();
                    data.lines()
                        .enumerate()
                        .take(rows as usize)
                        .for_each(|(row, line)| {
                            self.output.queue(cursor::MoveTo(0, row as u16)).unwrap();
                            self.output.queue(Print(line)).unwrap();
                        });
                }
            }
        }

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
