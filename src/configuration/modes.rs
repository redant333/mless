use regex::Regex;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer,
};

/// Structure describing a mode instance in the configuration file.
#[derive(Deserialize, Debug, PartialEq)]
pub struct Mode {
    /// Mode specific arguments that define this mode.
    #[serde(flatten)]
    pub args: ModeArgs,
    /// Hotkey to use during mode selection
    pub hotkey: char,
    /// Name to use during mode selection
    #[allow(dead_code)]
    pub name: String,
}

/// Arguments that specify the details of the mode.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "mode")]
pub enum ModeArgs {
    #[serde(rename = "regex")]
    RegexMode(RegexArgs),
}

/// Arguments for [crate::modes::RegexMode].
#[derive(Deserialize, Debug)]
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
                        Unexpected::Str(&regex_string),
                        &"a valid regular expression",
                    ))
                }
            }
        }

        Ok(regexes)
    }
}

impl PartialEq for RegexArgs {
    fn eq(&self, other: &Self) -> bool {
        if self.regexes.len() != other.regexes.len() {
            return false;
        }

        self.regexes
            .iter()
            .zip(other.regexes.iter())
            .all(|(regex1, regex2)| regex1.to_string() == regex2.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn regex_mode_can_be_deserialized() {
        let string = "
            mode: regex
            hotkey: r
            name: default
            regexes:
                - regex1
                - regex2
        ";

        let Mode { args, hotkey, name } = serde_yaml::from_str(string).unwrap();

        let ModeArgs::RegexMode(regex_args) = args;

        assert_eq!(hotkey, 'r');
        assert_eq!(name, "default");
        assert_eq!(regex_args.regexes[0].as_str(), "regex1");
        assert_eq!(regex_args.regexes[1].as_str(), "regex2");
    }

    #[test]
    fn deserialization_fails_if_no_regexes_are_provided() {
        let string = "
            mode: regex
            regexes: []
        ";

        let result = serde_yaml::from_str::<Mode>(string);
        result.unwrap_err();
    }

    #[test]
    fn deserialization_fails_if_invalid_regex_is_provided() {
        let string = "
            mode: regex
            regexes:
                - x[
        ";

        let result = serde_yaml::from_str::<Mode>(string);
        result.unwrap_err();
    }

    #[test_case(vec![], vec![], true)]
    #[test_case(vec![Regex::new(".+").unwrap()], vec![Regex::new(".+").unwrap()], true)]
    #[test_case(vec![Regex::new(".+").unwrap()], vec![], false)]
    fn equals_returns_expected_value(
        regexes1: Vec<Regex>,
        regexes2: Vec<Regex>,
        expected_equal: bool,
    ) {
        let args1 = RegexArgs { regexes: regexes1 };
        let args2 = RegexArgs { regexes: regexes2 };

        let equal = args1 == args2;
        assert_eq!(equal, expected_equal);
    }
}
