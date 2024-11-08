use std::fs::File;

use super::modes;
use regex::Regex;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
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
    #[serde(deserialize_with = "Config::validate_hint_characters")]
    pub hint_characters: String,
    /// List of modes that the user can use.
    ///
    /// Note that it is possible to have multiple instances of the same
    /// mode with different arguments. See [modes::Mode]
    #[serde(default = "Config::default_modes")]
    #[serde(deserialize_with = "Config::validate_modes")]
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
        "fdsajkl;weiocmruvnghqpxztyb".into()
    }

    fn validate_hint_characters<'de, D>(d: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hint_chars = String::deserialize(d)?;

        if !hint_chars.is_empty() {
            return Ok(hint_chars);
        }

        Err(de::Error::invalid_value(
            Unexpected::Str(&hint_chars),
            &"contain at least one character",
        ))
    }

    fn validate_modes<'de, D>(d: D) -> Result<Vec<modes::Mode>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let modes = Vec::<modes::Mode>::deserialize(d)?;

        if !modes.is_empty() {
            return Ok(modes);
        }

        Err(de::Error::invalid_value(
            Unexpected::Seq,
            &"at least one mode",
        ))
    }

    fn default_modes() -> Vec<modes::Mode> {
        vec![modes::Mode {
            hotkey: 'g',
            description: "General".to_string(),
            args: modes::ModeArgs::RegexMode(modes::RegexArgs {
                // Hardcoded value that is verified to work
                #[allow(clippy::unwrap_used)]
                regexes: vec![Regex::new(r"[\w._\-~/]{4,}").unwrap()],
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
mod tests {
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

    #[test]
    fn hint_characters_deserialization_returns_error_when_empty() {
        let result = serde_yaml::from_str::<Config>("hint_characters: ''");
        result.unwrap_err();
    }

    #[test]
    fn modes_deserialization_returns_error_when_empty() {
        let result = serde_yaml::from_str::<Config>("modes: []");
        result.unwrap_err();
    }
}
