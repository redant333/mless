mod configuration;
mod input_handler;
mod modes;
mod renderer;

use clap::Parser;
use configuration::Config;
use crossterm::event::read;
use input_handler::{Action, InputHandler};
use modes::{Mode, RegexMode};
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
    let mut renderer = Renderer {
        output: io::stdout(),
    };
    let mut current_mode = RegexMode {};

    let input_text = match args.file {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => io::stdin().lines().map(|line| line.unwrap()).collect(),
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
        let draw_instructions = current_mode.get_draw_instructions();
        renderer.render(&input_text, &draw_instructions);

        let action = match read() {
            Ok(event) => input_handler.get_action(event),
            _ => None,
        };

        let _mode_action = match action {
            Some(Action::Exit) => break,
            Some(Action::ForwardKeyPress(keypress)) => current_mode.handle_key_press(keypress),
            None => None,
        };
    }

    renderer.uninitialize_terminal().unwrap_or_else(|error| {
        eprintln!("Could not uninitialize the terminal: {}", error);
        eprintln!("Your terminal might start behaving incorrectly");
        exit(EXIT_ERROR);
    });

    exit(EXIT_SUCCESS);
}
