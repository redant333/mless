use std::fs::File;

use super::modes;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    ParseError { source: serde_yaml::Error },
}

/// The main configuration struct representing the whole configuration
/// file.
///
/// All of its fields have default values to enable starting without
/// any config specified and to enable config files to override only
/// some of the fields.
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    /// Characters that can be used by structs implementing [modes::Mode]
    /// trait.
    #[serde(default = "Config::default_hint_characters")]
    pub hint_characters: String,
    /// List of modes that the user can use.
    ///
    /// Note that it is possible to have multiple instances of the same
    /// mode with different arguments. See [modes::Mode]
    #[serde(default = "Config::default_modes")]
    pub modes: Vec<modes::Mode>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hint_characters: Config::default_hint_characters(),
            modes: Config::default_modes(),
        }
    }
}

impl Config {
    fn default_hint_characters() -> String {
        "qwertyuiopasdfghjklzxcvbnm".into()
    }

    fn default_modes() -> Vec<modes::Mode> {
        vec![modes::Mode {
            hotkey: 'w',
            description: "Select words separated by space".to_string(),
            args: modes::ModeArgs::RegexMode(modes::RegexArgs {
                regexes: vec![r"[a-zA-Z]{5,}".to_string()],
            }),
        }]
    }
}

impl TryFrom<File> for Config {
    type Error = Error;

    fn try_from(file: File) -> Result<Self, Self::Error> {
        let config = serde_yaml::from_reader(file) //
            .context(ParseSnafu {})?;

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_be_deserialized_from_empty_string() {
        serde_yaml::from_str::<Config>("").unwrap();
    }

    #[test]
    fn can_be_deserialized_from_partial_string() {
        let config: Config = serde_yaml::from_str("hint_characters: asdf").unwrap();

        assert_eq!(config.hint_characters, "asdf");
    }

    #[test]
    fn default_config_can_be_serialized() {
        let config = Config {
            ..Default::default()
        };
        let serialized = serde_yaml::to_string(&config).unwrap();

        assert!(!serialized.is_empty());
    }
}
