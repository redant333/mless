mod configuration;
mod render;

use clap::Parser;
use configuration::Config;
use crossterm::{cursor, terminal};
use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    QueueableCommand,
};
use render::Renderer;
use std::io;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// File to select the text from. Omit to use standard input
    file: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();
    let _config = Config {};

    let (cols, rows) = terminal::size().unwrap();
    let mut renderer = Renderer {
        output: io::stdout(),
        rows,
        cols,
    };

    let mut input = match args.file {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => io::stdin().lines().map(|line| line.unwrap()).collect(),
    };

    renderer.output.queue(cursor::Hide).unwrap();
    enable_raw_mode().unwrap();
    loop {
        renderer.render(&input);

        if let Event::Key(event) = read().unwrap() {
            if let KeyCode::Char(c) = event.code {
                match c {
                    'q' => break,
                    _ => input.push(c),
                }
            }
        }
    }
    renderer.output.queue(cursor::Show).unwrap();
    disable_raw_mode().unwrap();
}
