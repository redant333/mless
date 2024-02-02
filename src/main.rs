mod configuration;
mod input_handling;
mod render;

use clap::Parser;
use configuration::Config;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, terminal};
use crossterm::{
    event::read,
    terminal::{disable_raw_mode, enable_raw_mode},
    QueueableCommand,
};
use input_handling::{Action, InputHandler};
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
    let config = Config {
        ..Default::default()
    };

    let input_handler = InputHandler::from_config(&config);
    let input_text = match args.file {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => io::stdin().lines().map(|line| line.unwrap()).collect(),
    };

    let (cols, rows) = terminal::size().unwrap();
    let mut renderer = Renderer {
        output: io::stdout(),
        rows,
        cols,
    };

    renderer
        .output
        .queue(cursor::Hide)
        .unwrap()
        .queue(EnterAlternateScreen)
        .unwrap();
    enable_raw_mode().unwrap();

    loop {
        renderer.render(&input_text);

        let action = match read() {
            Ok(event) => input_handler.get_action(event),
            _ => None,
        };

        // Will eventually contain matching for all different actions
        #[allow(clippy::single_match)]
        match action {
            Some(Action::Exit) => break,
            _ => (),
        };
    }

    renderer
        .output
        .queue(LeaveAlternateScreen)
        .unwrap()
        .queue(cursor::Show)
        .unwrap();
    disable_raw_mode().unwrap();
}
