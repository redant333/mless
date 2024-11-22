//! Initialization, main loop and similar.
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
};

use crossterm::event::read;
use log::{debug, info};
use snafu::ResultExt;

use crate::{
    app::configuration_handling::load_config,
    configuration::ModeArgs,
    error::{CouldNotReadInputSnafu, RunError, TerminalHandlingSnafu, TtyOpenSnafu},
    hints::HintPoolGenerator,
    input_handler::{Action, InputHandler},
    logging::initialize_logging,
    modes::{Mode, ModeEvent, RegexMode},
    rendering::{DrawInstruction, Renderer},
};

use crate::args::Args;

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

fn run_main_loop(
    input_handler: InputHandler,
    initial_mode: RegexMode,
    renderer: &mut Renderer<File>,
    input_text: String,
) -> Result<String, RunError> {
    let mut current_mode = initial_mode;

    // Make sure the data is rendered as early as possible to avoid blinking
    renderer.render(&input_text, &[DrawInstruction::Data])?;

    info!("Starting the loop");
    loop {
        let draw_instructions = current_mode.get_draw_instructions();
        renderer.render(&input_text, &draw_instructions)?;

        let action = match read() {
            Ok(event) => {
                debug!("Got event {:?}", event);
                input_handler.get_action(event)
            }
            _ => None,
        };

        debug!("Got input handler action {:?}", action);

        let mode_action = match action {
            Some(Action::Exit) => return Ok("".to_string()),
            Some(Action::ForwardKeyPress(keypress)) => current_mode.handle_key_press(keypress),
            None => None,
        };

        debug!("Got mode action {:?}", mode_action);

        // The enum will get more variants, so make it a match from the start
        #[allow(clippy::single_match)]
        match mode_action {
            Some(ModeEvent::TextSelected(text)) => {
                return Ok(text);
            }
            None => (),
        }
    }
}

pub fn run(args: Args) -> Result<String, RunError> {
    initialize_logging()?;
    info!("Initializing");

    let config = load_config(args.config)?;

    let input_handler = InputHandler::from_config(&config);
    let mut renderer = create_renderer()?;

    let input_text = match args.file {
        Some(path) => {
            std::fs::read_to_string(path) //
                .context(CouldNotReadInputSnafu {})?
        }
        None => {
            let mut ret = "".to_string();
            io::stdin()
                .read_to_string(&mut ret) //
                .context(CouldNotReadInputSnafu {})?;
            ret
        }
    };

    let hint_generator = Box::new(HintPoolGenerator::new(&config.hint_characters));

    let ModeArgs::RegexMode(args) = &config.modes[0].args;
    let initial_mode = RegexMode::new(&input_text, args, hint_generator)?;

    renderer
        .initialize_terminal()
        .context(TerminalHandlingSnafu {
            operation: "initialize",
        })?;

    let ret = run_main_loop(input_handler, initial_mode, &mut renderer, input_text);

    renderer
        .uninitialize_terminal()
        .context(TerminalHandlingSnafu {
            operation: "uninitialize",
        })?;

    ret
}
