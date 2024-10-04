//! Error enum.
use crate::configuration;
use snafu::prelude::*;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
/// Main error enum for the application.
pub enum RunError {
    /// Could not open config file.
    #[snafu(display("Could not open config file {}\n{}", path.display(), source))]
    ConfigOpen {
        /// The source error that caused this [RunError].
        source: io::Error,
        /// Path of the config file whose opening failed.
        path: PathBuf,
    },

    /// The config file could be opened and read, but it's contents are not as expected.
    #[snafu(display("Could not parse config file {}\n{}", path.display(), source))]
    ConfigParse {
        /// The source error that caused this [RunError].
        source: configuration::Error,
        /// Path of the config file whose parsing failed.
        path: PathBuf,
    },

    /// Could not open the device used to draw the interface.
    #[snafu(display("Could not open /dev/tty for writing\n{}", source))]
    TtyOpen {
        /// The source error that caused this [RunError].
        source: io::Error,
    },

    /// Could not initialize/uninitialize the terminal.
    /// There is no guarantee that the terminal was left in a correct state.
    #[snafu(display("Could not {operation} the terminal\n{source}"))]
    TerminalHandling {
        /// The source error that caused this [RunError].
        source: io::Error,
        /// The operation that failed. Should be "initialize" or "uninitialize".
        operation: String,
    },

    /// Could not initialize the logging system.
    #[snafu(display("Could not start logging to {}\n{}", path, source))]
    LoggingStart {
        /// The source error that caused this [RunError].
        source: io::Error,
        /// The path of the file that to which logging was attempted.
        path: String,
    },

    /// Invalid regular expression provided.
    #[snafu(display("Invalid regular expression\n{}", source))]
    InvalidRegex {
        /// The source error that caused this [RunError].
        source: regex::Error,
    },

    /// Error duing IO operations that doesn't fit any of the more specific categories.
    #[snafu(display("IO error\n{}", source))]
    IoError {
        /// The source error that caused this [RunError].
        source: io::Error,
    },

    /// The input, file or stdin, could not be read.
    #[snafu(display("Could not read input\n{}", source))]
    CouldNotReadInput {
        /// The source error that caused this [RunError].
        source: io::Error,
    },
}
