use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Mode {
    pub hotkey: char,
    pub description: String,
    #[serde(flatten)]
    pub args: ModeArgs,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "mode")]
pub enum ModeArgs {
    #[serde(rename = "regex")]
    RegexMode(RegexArgs),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegexArgs {
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
