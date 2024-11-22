use regex::Regex;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer,
};

/// Structure describing a mode instance in the configuration file.
#[derive(Deserialize, Debug)]
pub struct Mode {
    /// Mode specific arguments that define this mode.
    #[serde(flatten)]
    pub args: ModeArgs,
}

/// Arguments that specify the details of the mode.
#[derive(Deserialize, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_mode_can_be_deserialized() {
        let string = "
            mode: regex
            regexes:
                - regex1
                - regex2
        ";

        let Mode { args } = serde_yaml::from_str(string).unwrap();

        let ModeArgs::RegexMode(regex_args) = args;

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
}
