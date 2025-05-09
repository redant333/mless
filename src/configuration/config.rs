use std::{collections::HashSet, fs::File};

use super::{deserialize_color, modes, DEFAULT_CONFIG_FILE};
use crossterm::style::Color;
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
#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    /// Characters that can be used by structs implementing [modes::Mode]
    /// trait.
    #[serde(default = "Config::default_hint_characters")]
    #[serde(deserialize_with = "Config::validate_hint_characters")]
    pub hint_characters: String,

    /// Foreground color for hints during selection.
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_hint_fg")]
    pub hint_fg: Color,

    /// Background color for hints during selection.
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_hint_bg")]
    pub hint_bg: Color,

    /// Foreground color for highlights during selection.
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_highlight_fg")]
    pub highlight_fg: Color,

    /// Background color for highlights during selection.
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_highlight_bg")]
    pub highlight_bg: Color,

    /// Foreground color of the mode switching divider character
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_mode_switch_divider_fg")]
    pub mode_switch_divider_fg: Color,

    /// Foreground color of mode hotkeys displayed during mode switching
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_mode_switch_hotkey_fg")]
    pub mode_switch_hotkey_fg: Color,

    /// Foreground color of mode names displayed during mode switching
    #[serde(deserialize_with = "deserialize_color")]
    #[serde(default = "Config::default_mode_switch_mode_name_fg")]
    pub mode_switch_mode_name_fg: Color,

    /// Mode switching dialog width including the divider
    #[serde(default = "Config::default_mode_switch_width")]
    pub mode_switch_width: usize,

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

    fn default_hint_fg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;232").unwrap()
    }

    fn default_hint_bg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;208").unwrap()
    }

    fn default_highlight_fg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;232").unwrap()
    }

    fn default_highlight_bg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;252").unwrap()
    }

    fn default_mode_switch_divider_fg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;208").unwrap()
    }

    fn default_mode_switch_hotkey_fg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;208").unwrap()
    }

    fn default_mode_switch_mode_name_fg() -> Color {
        #[allow(clippy::unwrap_used)] // Parsing will always succeed for these literals
        Color::parse_ansi("5;252").unwrap()
    }

    fn default_mode_switch_width() -> usize {
        25
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
    // This is necessary to make sure that the user can omit some values in their
    // config and get the default values for the rest
    fn config_with_default_fields_equal_to_parsed_default_config() {
        let default_config = Config::default();
        let config_with_default_fields = serde_yaml::from_str::<Config>("").unwrap();

        assert_eq!(default_config, config_with_default_fields);
    }
}
