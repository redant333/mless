//! Definition of the available application arguments.
use clap::Parser;

const AFTER_HELP: &str = "
CONFIGURATION

    The behavior can be customized with a configuration file. The following
    locations, in this priority, are checked:

     - File specified with --config argument
     - $XDG_CONFIG_HOME/mless/mless.yaml
     - $HOME/.config/mless/mless.yaml
     - $HOME/.mless.yaml

    If none of them can be found, the default values that can be seen by
    running with --show-default-config are used.
    If a setting is not present in the config file, the default value is
    used.

COLORS

    Whenever a color needs to be specified in the configuration, the
    following three ways of specifying it are supported:

     - One of the following words specifying a color from the 16 colors set:
       black, dark_grey, red, dark_red, green, dark_green,
       yellow, dark_yellow, blue, dark_blue, magenta,
       dark_magenta, cyan, dark_cyan, white, grey.
     - A color from the 256 colors set in format \"5;<color_index>\", e.g.
       \"5;202\" for orange.
     - An RGB color in format \"2;<r>;<g>;<b>\", e.g. \"2;255;255;0\" for
       yellow.

     Note that the ability to render different colors depends on your
     terminal emulator and configuration.
";

#[derive(Debug, Parser)]
#[command(author, version, about, after_help=AFTER_HELP)]
pub struct Args {
    /// File to select the text from. Omit to use standard input.
    pub file: Option<std::path::PathBuf>,

    /// Config file to read.
    #[arg(short, long, value_name = "CONFIG_FILE")]
    pub config: Option<std::path::PathBuf>,

    /// Show the default config with documentation file and exit.
    #[arg(long, action)]
    pub show_default_config: bool,

    /// Start in selection mode with hotkey MODE instead of the first one specified in config
    #[arg(short = 'm', long = "start-in-mode", value_name = "MODE")]
    pub start_in_mode: Option<char>,
}
