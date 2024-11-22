//! Definition of the available application arguments.
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// File to select the text from. Omit to use standard input.
    pub file: Option<std::path::PathBuf>,
    /// Config file to read.
    #[arg(short, long, value_name = "CONFIG_FILE")]
    pub config: Option<std::path::PathBuf>,
}
