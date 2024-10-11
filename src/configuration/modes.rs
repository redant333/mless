use regex::Regex;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Structure describing a mode instance in the configuration file.
#[derive(Serialize, Deserialize, Debug)]
pub struct Mode {
    /// The hotkey that is used to enter the mode.
    pub hotkey: char,
    /// Description of the mode.
    ///
    /// This is only used in situations when more information needs to
    /// be presented about the mode instance.
    /// Defaults to empty string if not provided.
    #[serde(default)]
    pub description: String,
    /// Mode specific arguments that define this mode.
    #[serde(flatten)]
    pub args: ModeArgs,
}

/// Arguments that specify the details of the mode.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "mode")]
pub enum ModeArgs {
    #[serde(rename = "regex")]
    RegexMode(RegexArgs),
}

/// Arguments for [crate::modes::RegexMode].
#[derive(Serialize, Deserialize, Debug)]
pub struct RegexArgs {
    /// The list of regexes that the mode will use for selections.
    #[serde(deserialize_with = "RegexArgs::deserialize_regexes")]
    #[serde(serialize_with = "RegexArgs::serialize_regexes")]
    pub regexes: Vec<Regex>,
}

impl RegexArgs {
    fn deserialize_regexes<'de, D>(d: D) -> Result<Vec<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regex_strings = Vec::<String>::deserialize(d)?;
        if regex_strings.is_empty() {
            return Err(de::Error::invalid_value(
                Unexpected::Seq,
                &"a sequence of one or more valid regular expressions",
            ));
        }

        let mut regexes = vec![];

        for regex_string in regex_strings.into_iter() {
            let regex = Regex::new(&regex_string);

            match regex {
                Ok(regex) => regexes.push(regex),
                Err(_) => {
                    return Err(de::Error::invalid_value(
                        Unexpected::Seq,
                        &"a valid regular expression",
                    ))
                }
            }
        }

        Ok(regexes)
    }

    fn serialize_regexes<S>(regexes: &[Regex], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let regex_strings: Vec<&str> = regexes.iter().map(|regex| regex.as_str()).collect();
        regex_strings.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_mode_can_be_deserialized() {
        let string = "
            mode: regex
            description: Select things and stuff
            hotkey: x
            regexes:
                - regex1
                - regex2
        ";

        let Mode {
            hotkey,
            description,
            args,
        } = serde_yaml::from_str(string).unwrap();

        assert_eq!(hotkey, 'x');
        assert_eq!(description, "Select things and stuff");

        let ModeArgs::RegexMode(regex_args) = args;

        assert_eq!(regex_args.regexes[0].as_str(), "regex1");
        assert_eq!(regex_args.regexes[1].as_str(), "regex2");
    }

    #[test]
    fn deserialization_returns_error_when_hotkey_too_long() {
        let string = "
            mode: regex
            description: Select things and stuff
            hotkey: xy
            regexes:
                - regex1
                - regex2
        ";

        let result = serde_yaml::from_str::<Mode>(string);
        result.unwrap_err();
    }

    #[test]
    fn description_defaults_to_empty_string_if_not_provided() {
        let string = "
            mode: regex
            hotkey: x
            regexes:
                - regex1
                - regex2
        ";

        let Mode { description, .. } = serde_yaml::from_str(string).unwrap();
        assert_eq!(description, "");
    }

    #[test]
    fn deserialization_fails_if_no_regexes_are_provided() {
        let string = "
            mode: regex
            hotkey: x
            regexes: []
        ";

        let result = serde_yaml::from_str::<Mode>(string);
        result.unwrap_err();
    }

    #[test]
    fn deserialization_fails_if_invalid_regex_is_provided() {
        let string = "
            mode: regex
            hotkey: x
            regexes:
                - x[
        ";

        let result = serde_yaml::from_str::<Mode>(string);
        result.unwrap_err();
    }
}
