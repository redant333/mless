//! Structs and functions for dealing with config files.
mod config;
pub use config::Config;
pub use config::Error;

mod modes;
pub use modes::Mode;
pub use modes::ModeArgs;
pub use modes::RegexArgs;

pub const DEFAULT_CONFIG_FILE: &str = include_str!("default_config.yaml");
