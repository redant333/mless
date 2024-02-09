mod configuration;
mod input_handler;
mod renderer;

use clap::Parser;
use configuration::Config;
use crossterm::event::read;
use crossterm::terminal;
use input_handler::{Action, InputHandler};
use renderer::Renderer;
use std::io;
use std::process::exit;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// File to select the text from. Omit to use standard input
    file: Option<std::path::PathBuf>,
}

const EXIT_ERROR: i32 = -1;
const EXIT_SUCCESS: i32 = 0;

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

    renderer.initialize_terminal().unwrap_or_else(|error| {
        eprintln!("Could not initialize the terminal: {}", error);
        // Do a best effort to reset the terminal
        renderer.uninitialize_terminal().unwrap_or_else(|error| {
            eprintln!(
                "Could not recover the terminal after failed initialization: {}",
                error
            );
            eprintln!("Your terminal might start behaving incorrectly");
        });
        exit(EXIT_ERROR);
    });

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

    renderer.uninitialize_terminal().unwrap_or_else(|error| {
        eprintln!("Could not uninitialize the terminal: {}", error);
        eprintln!("Your terminal might start behaving incorrectly");
        exit(EXIT_ERROR);
    });

    exit(EXIT_SUCCESS);
}
