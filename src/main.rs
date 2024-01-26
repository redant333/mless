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
struct Args {}

fn main() {
    let _args = Args::parse();
    let _config = Config {};

    let (cols, rows) = terminal::size().unwrap();
    let mut renderer = Renderer {
        output: io::stdout(),
        rows,
        cols,
    };

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

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
