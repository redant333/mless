use crossterm::style::Color;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer,
};

pub fn deserialize_color<'de, D>(d: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let color_string = String::deserialize(d)?;

    match string_to_color(&color_string) {
        Some(color) => Ok(color),
        None => Err(de::Error::invalid_value(
            Unexpected::Str(&color_string),
            &"be an ANSI color like 5;252, RGB color like 2;50;60;70 or one of black, \
                  dark_grey, red, dark_red, green, dark_green, yellow, dark_yellow, blue, \
                  dark_blue, magenta, dark_magenta, cyan, dark_cyan, white, grey",
        )),
    }
}

/// Attempt converting the given string containing a color name or ANSI code into a color.
fn string_to_color(string: &str) -> Option<Color> {
    // First attempt parsing it as a named color, e.g. dark_red
    if let Ok(color) = string.try_into() {
        return Some(color);
    }

    // Otherwise consider it an ANSI input
    Color::parse_ansi(string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", None; "for empty string")]
    #[test_case("black", Some(Color::Black); "for input 'black'")]
    #[test_case("dark_grey", Some(Color::DarkGrey); "for input 'dark_grey'")]
    #[test_case("red", Some(Color::Red); "for input 'red'")]
    #[test_case("dark_red", Some(Color::DarkRed); "for input 'dark_red'")]
    #[test_case("green", Some(Color::Green); "for input 'green'")]
    #[test_case("dark_green", Some(Color::DarkGreen); "for input 'dark_green'")]
    #[test_case("yellow", Some(Color::Yellow); "for input 'yellow'")]
    #[test_case("dark_yellow", Some(Color::DarkYellow); "for input 'dark_yellow'")]
    #[test_case("blue", Some(Color::Blue); "for input 'blue'")]
    #[test_case("dark_blue", Some(Color::DarkBlue); "for input 'dark_blue'")]
    #[test_case("magenta", Some(Color::Magenta); "for input 'magenta'")]
    #[test_case("dark_magenta", Some(Color::DarkMagenta); "for input 'dark_magenta'")]
    #[test_case("cyan", Some(Color::Cyan); "for input 'cyan'")]
    #[test_case("dark_cyan", Some(Color::DarkCyan); "for input 'dark_cyan'")]
    #[test_case("white", Some(Color::White); "for input 'white'")]
    #[test_case("grey", Some(Color::Grey); "for input 'grey'")]
    #[test_case("5;232", Some(Color::AnsiValue(232)); "for ANSI input")]
    #[test_case("2;50;60;70", Some(Color::Rgb{r:50, g:60, b:70}); "for RGB input")]
    fn string_to_color_returns_expected_value(string: &str, expected: Option<Color>) {
        assert_eq!(string_to_color(string), expected);
    }
}
