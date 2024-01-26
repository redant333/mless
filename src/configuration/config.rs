use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default = "Config::default_hint_characters")]
    pub hint_characters: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hint_characters: Config::default_hint_characters(),
        }
    }
}

impl Config {
    fn default_hint_characters() -> String {
        "qwertyuiopasdfghjklzxcvbnm".into()
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
    fn can_be_deserialized_from_full_string() {
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
