//! Initialization, main loop and similar.
use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, Read},
    ops::Deref,
};

use crossterm::{event::read, terminal};
use log::{debug, info};
use snafu::ResultExt;

use crate::{
    app::configuration_handling::{get_config_file_location, load_config},
    configuration::{self, ModeArgs},
    error::{CouldNotReadInputSnafu, RunError, TerminalHandlingSnafu, TtyOpenSnafu},
    hints::{HintGenerator, HintPoolGenerator},
    input_handler::{Action, InputHandler},
    logging::initialize_logging,
    modes::{Mode, ModeEvent, RegexMode},
    pager::get_page,
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

fn create_mode(
    input_text: &str,
    hint_generator: &dyn HintGenerator,
    args: &configuration::ModeArgs,
) -> Result<RegexMode, RunError> {
    let ModeArgs::RegexMode(args) = args;
    let mode = RegexMode::new(input_text, args, hint_generator)?;

    Ok(mode)
}

fn get_input_text(args: &Args) -> Result<String, RunError> {
    let input_text = match &args.file {
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
    Ok(input_text)
}

fn get_input_page(input_text: &str) -> Result<String, RunError> {
    let (cols, rows) = terminal::size() //
        .context(TerminalHandlingSnafu {
            operation: "get size",
        })?;

    let mut input_buffer = BufReader::new(input_text.as_bytes());
    let input_page = get_page(&mut input_buffer, rows as usize, cols as usize);

    Ok(input_page)
}

fn run_main_loop(
    input_handler: InputHandler,
    hint_generator: &dyn HintGenerator,
    modes: &[configuration::Mode],
    renderer: &mut Renderer<File>,
    input_text: String,
) -> Result<String, RunError> {
    let mut input_page = get_input_page(&input_text)?;
    let mut current_mode = create_mode(&input_text, hint_generator, &modes[0].args)?;

    // Make sure the data is rendered as early as possible to avoid blinking
    renderer.render(&input_page, &[DrawInstruction::Data])?;

    info!("Starting the loop");
    loop {
        let draw_instructions = current_mode.get_draw_instructions();
        renderer.render(&input_page, &draw_instructions)?;

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
            Some(Action::Resize) => {
                input_page = get_input_page(&input_text)?;
                current_mode = create_mode(&input_page, hint_generator, &modes[0].args)?;
                None
            }
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

    let config_path = match &args.config {
        Some(path) => Some(path.clone()),
        None => get_config_file_location(),
    };
    let config = load_config(config_path)?;

    let input_handler = InputHandler::from_config(&config);
    let mut renderer = create_renderer()?;

    // This approach is not ideal since it reads the whole input text
    // while only using one screen of text but it should be OK for now
    let input_text = get_input_text(&args)?;

    let hint_generator: Box<dyn HintGenerator> =
        Box::new(HintPoolGenerator::new(&config.hint_characters));

    renderer
        .initialize_terminal()
        .context(TerminalHandlingSnafu {
            operation: "initialize",
        })?;

    let ret = run_main_loop(
        input_handler,
        hint_generator.deref(),
        &config.modes,
        &mut renderer,
        input_text,
    );

    renderer
        .uninitialize_terminal()
        .context(TerminalHandlingSnafu {
            operation: "uninitialize",
        })?;

    ret
}
