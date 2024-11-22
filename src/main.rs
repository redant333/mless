//! # Architecture
//! The diagram below shows the relationship between the most important
//! components of the system.
#![doc = include_str!("./docs/architecture_diagram.svg")]
//!
//! For convenience, here are links to the mentioned structs/traits/functions:
//! - [input_handler::InputHandler]
//! - [input_handler::Action]
//! - [app::run]
//! - [configuration::Config]
//! - [modes::Mode]
//! - [rendering::Renderer]
//! - [rendering::DrawInstruction]
mod app;
mod args;
mod configuration;
mod error;
mod hints;
mod input_handler;
mod logging;
mod modes;
mod rendering;

use std::process::exit;

use app::run;
use args::Args;
use clap::Parser;

fn main() {
    const EXIT_ERROR: i32 = -1;
    const EXIT_SUCCESS: i32 = 0;

    let args = Args::parse();

    if args.show_default_config {
        println!("{}", configuration::DEFAULT_CONFIG_FILE);
        exit(EXIT_SUCCESS);
    }

    match run(args) {
        Ok(selection) => {
            print!("{}", selection);
            exit(EXIT_SUCCESS);
        }
        Err(error) => {
            eprintln!("{}", error);
            exit(EXIT_ERROR);
        }
    }
}
