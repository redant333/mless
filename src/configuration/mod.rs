//! Structs and functions for dealing with config files.
mod config;
pub use config::Config;
pub use config::Error;

mod modes;
pub use modes::ModeArgs;
pub use modes::RegexArgs;
