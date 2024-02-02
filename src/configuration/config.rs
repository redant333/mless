use super::modes;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default = "Config::default_hint_characters")]
    pub hint_characters: String,
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
                regexes: vec![r#"[^ ]+"#.to_string()],
            }),
        }]
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
