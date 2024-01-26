use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_be_deserialized_from_empty_string() {
        serde_yaml::from_str::<Config>("").unwrap();
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
