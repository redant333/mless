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
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use std::process::exit;

use crate::configuration::ModeArgs;
use crate::hints::HintPoolGenerator;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum RunError {
    #[snafu(display("Could not open config file {}\n{}", path.display(), source))]
    ConfigOpen { source: io::Error, path: PathBuf },

    #[snafu(display("Could not parse config file {}\n{}", path.display(), source))]
    ConfigParse {
        source: configuration::Error,
        path: PathBuf,
    },

    #[snafu(display("Could not open /dev/tty for writing\n{}", source))]
    TtyOpen { source: io::Error },
}

// TODO Replace all panics, unwraps and similar with something
// that will not crash the program. It is important not to crash
// in order to uninitialize the terminal and leave it in a good
// state.

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// File to select the text from. Omit to use standard input.
    file: Option<std::path::PathBuf>,
    /// Config file to read.
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<std::path::PathBuf>,
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
/// Load the [Config] from the given path. If path is [None], the default
/// value for [Config] is returned.
fn load_config(path: Option<PathBuf>) -> Result<Config, RunError> {
    // TODO Validate the configuration before continuing.
    // It is possible that some things will be validated automatically,
    // due to used types, but at least some things need to be validated
    // manually.
    if let Some(path) = path {
        let file = File::open(path.clone()) //
            .context(ConfigOpenSnafu { path: path.clone() })?;
        let config = Config::try_from(file) //
            .context(ConfigParseSnafu { path })?;

        return Ok(config);
    }

    Ok(Config {
        ..Default::default()
    })
}

fn create_renderer() -> Result<Renderer<File>, RunError> {
    // Perform rendering to /dev/tty to enable piping of the output
    let output_path = "/dev/tty";

    let tty = OpenOptions::new()
        .append(true)
        .open(output_path)
        .context(TtyOpenSnafu {})?;

    let renderer = Renderer { output: tty };

    Ok(renderer)
}

fn run() -> Result<(), RunError> {
    initialize_logging();
    info!("Initializing");

    let args = Args::parse();
    let config = load_config(args.config)?;

    let input_handler = InputHandler::from_config(&config);
    let mut renderer = create_renderer()?;

    let input_text = match args.file {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => io::stdin()
            .lines()
            .map(|line| line.unwrap() + "\n")
            .collect(),
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
            Ok(event) => {
                debug!("Got event {:?}", event);
                input_handler.get_action(event)
            }
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

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        exit(EXIT_ERROR);
    }

    exit(EXIT_SUCCESS);
}
