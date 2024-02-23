use serde::{Deserialize, Serialize};

/// Structure describing a mode instance in the configuration file.
#[derive(Serialize, Deserialize, Debug)]
pub struct Mode {
    /// The hotkey that is used to enter the mode.
    pub hotkey: char,
    /// Description of the mode.
    ///
    /// This is only used in situations when more information needs to
    /// be presented about the mode instance.
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
    pub regexes: Vec<String>,
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

        assert_eq!(regex_args.regexes[0], "regex1");
        assert_eq!(regex_args.regexes[1], "regex2");
    }
}
