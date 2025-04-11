use std::{collections::HashSet, fs::File};

use super::{modes, DEFAULT_CONFIG_FILE};
use regex::Regex;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer,
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
#[derive(Deserialize, Debug)]
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
        // Can only fail if the config file has unexpected structure.
        // DEFAULT_CONFIG_FILE is a known static string.
        #[allow(clippy::unwrap_used)]
        serde_yaml::from_str(DEFAULT_CONFIG_FILE).unwrap()
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

        if modes.is_empty() {
            return Err(de::Error::invalid_value(
                Unexpected::Seq,
                &"at least one mode",
            ));
        }

        let hotkeys: HashSet<char> = modes.iter().map(|mode| mode.hotkey).collect();
        if modes.len() != hotkeys.len() {
            return Err(de::Error::invalid_value(
                Unexpected::Seq,
                &"all hotkeys different",
            ));
        }

        Ok(modes)
    }

    fn default_modes() -> Vec<modes::Mode> {
        vec![modes::Mode {
            args: modes::ModeArgs::RegexMode(modes::RegexArgs {
                // Hardcoded value that is verified to work
                #[allow(clippy::unwrap_used)]
                regexes: vec![Regex::new(r"[\w._\-~/]{4,}").unwrap()],
            }),
            hotkey: 'r',
            name: "default".to_string(),
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
    fn hint_characters_deserialization_returns_error_when_empty() {
        let result = serde_yaml::from_str::<Config>("hint_characters: ''");
        result.unwrap_err();
    }

    #[test]
    fn modes_deserialization_returns_error_when_empty() {
        let result = serde_yaml::from_str::<Config>("modes: []");
        result.unwrap_err();
    }

    #[test]
    fn modes_deserialization_returns_error_for_repeated_hotkey() {
        let result = serde_yaml::from_str::<Config>(
            "
        modes:
          - mode: regex
            hotkey: r
            regexes:
              - regex1
              - regex2
          - mode: regex
            hotkey: r
            regexes:
              - regex3
              - regex4
        ",
        );
        result.unwrap_err();
    }

    #[test]
    /// This is necessary to allow them to be omitted in the config
    /// without confusing the user.
    fn omitted_hint_characters_default_to_the_value_in_default_config() {
        let default_config = Config::default();
        let config_with_default_fields = serde_yaml::from_str::<Config>("").unwrap();

        assert!(default_config.hint_characters == config_with_default_fields.hint_characters);
    }
}
