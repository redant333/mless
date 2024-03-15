mod configuration;
mod hints;
mod input_handler;
mod modes;
mod renderer;

use clap::Parser;
use configuration::Config;
use crossterm::event::read;
use env_logger::Env;
use input_handler::{Action, InputHandler};
use log::{debug, info};
use modes::{Mode, ModeEvent, RegexMode};
use renderer::Renderer;
use std::fs::File;
use std::io;
use std::process::exit;

use crate::configuration::ModeArgs;
use crate::hints::HintPoolGenerator;

// TODO Replace all panics, unwraps and similar with something
// that will not crash the program. It is important not to crash
// in order to uninitialize the terminal and leave it in a good
// state.

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// File to select the text from. Omit to use standard input.
    file: Option<std::path::PathBuf>,
}

const EXIT_ERROR: i32 = -1;
const EXIT_SUCCESS: i32 = 0;

const LOG_PATH_ENV: &str = "MLESS_LOG";
const LOG_DEFAULT_LEVEL: &str = "debug";

fn initialize_logging() {
    let Ok(log_path) = std::env::var(LOG_PATH_ENV) else {
        return;
    };

    let msg = format!("Couldn't open {} for logging", log_path);
    let log_file = Box::new(File::create(log_path.clone()).expect(&msg));

    env_logger::Builder::from_env(Env::default().default_filter_or(LOG_DEFAULT_LEVEL))
        .target(env_logger::Target::Pipe(log_file))
        .init();

    info!("Logging into {}", log_path);
}

fn main() {
    initialize_logging();
    info!("Initializing");

    let args = Args::parse();
    // TODO Validate the configuration before continuing.
    // It is possible that some things will be validated automatically,
    // due to used types, but at least some things need to be validated
    // manually.
    let config = Config {
        ..Default::default()
    };

    let input_handler = InputHandler::from_config(&config);
    let mut renderer = Renderer {
        output: io::stdout(),
    };

    let input_text = match args.file {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => io::stdin().lines().map(|line| line.unwrap()).collect(),
    };

    let hint_generator = Box::new(HintPoolGenerator::new(&config.hint_characters));

    let ModeArgs::RegexMode(args) = &config.modes[0].args;
    let mut current_mode = RegexMode::new(&input_text, args, hint_generator);

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

    let mut return_text = String::new();

    info!("Starting the loop");
    loop {
        let draw_instructions = current_mode.get_draw_instructions();
        renderer.render(&input_text, &draw_instructions);

        let action = match read() {
            Ok(event) => input_handler.get_action(event),
            _ => None,
        };

        debug!("Got input handler action {:?}", action);

        let mode_action = match action {
            Some(Action::Exit) => break,
            Some(Action::ForwardKeyPress(keypress)) => current_mode.handle_key_press(keypress),
            None => None,
        };

        debug!("Got mode action {:?}", mode_action);

        // The enum will get more variants, so make it a match from the start
        #[allow(clippy::single_match)]
        match mode_action {
            Some(ModeEvent::TextSelected(text)) => {
                return_text = text;
                break;
            }
            None => (),
        }
    }

    renderer.uninitialize_terminal().unwrap_or_else(|error| {
        eprintln!("Could not uninitialize the terminal: {}", error);
        eprintln!("Your terminal might start behaving incorrectly");
        exit(EXIT_ERROR);
    });

    print!("{}", return_text);

    exit(EXIT_SUCCESS);
}
