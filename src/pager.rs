use std::{io::BufRead, iter::once};

use log::warn;
use unicode_width::UnicodeWidthStr;

/// Clip the given line so that it fits into the given numbers of rows of the given width.
/// Note that this takes into account the fact that some characters, e.g. emojis, take up
/// two spaces when rendered.
///
/// Returns a tuple containing the clipped version of the line and the number of rows it
/// fills up.
fn clip_line(line: &str, rows: usize, row_width: usize) -> (String, usize) {
    let mut current_row_start = 0;
    let mut last_slice_to = 0;
    let mut current_row_index = 0;

    let substring_ends = line
        .char_indices()
        .map(|(char_index, _)| char_index)
        .chain(once(line.len()));

    for substring_to in substring_ends {
        let slice = &line[current_row_start..substring_to];

        if slice.width() > row_width {
            current_row_index += 1;
            current_row_start = last_slice_to;

            if current_row_index == rows {
                return (line[0..current_row_start].to_string(), current_row_index);
            }
        }

        last_slice_to = substring_to;
    }

    // If we didn't mange to full up all the given rows, just return the whole string
    // along with the number of rows we did manage to fill up.
    (line.to_string(), current_row_index + 1)
}

// Get largest substring from the source that can be rendered in the space of the given size.
#[allow(dead_code)]
pub fn get_page(source: &mut dyn BufRead, rows: usize, cols: usize) -> String {
    let mut output_lines = vec![];
    let mut output_rows_remaining = rows;

    for line in source.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                warn!("Could not read the whole input: {error}");
                break;
            }
        };

        let (line_clipped, line_rows) = clip_line(&line, output_rows_remaining, cols);

        output_lines.push(line_clipped);

        if line_rows <= output_rows_remaining {
            output_rows_remaining -= line_rows;
        } else {
            warn!("Line clipping produced {line_rows} when only {output_rows_remaining} remaining");
            break;
        }

        if output_rows_remaining == 0 {
            break;
        }
    }

    output_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;
    use test_case::test_case;

    #[test_case("", 10, 10, ""; "when_input_is_empty")]
    #[test_case("test\ntest", 10, 10, "test\ntest"; "when_input_shorter_than_page")]
    #[test_case("test1\ntest2\ntest3", 2, 10, "test1\ntest2"; "when_input_longer_than_page")]
    #[test_case("things and stuff\nstuff and things", 10, 10, "things and stuff\nstuff and things"; "when_input_wider_than_page")]
    #[test_case("things and stuff\nstuff and things", 3, 10, "things and stuff\nstuff and "; "when_input_longer_and_wider_than_page")]
    #[test_case("fläder väder", 2, 6, "fläder väder"; "when_input_contains_non_ascii_characters")]
    #[test_case("😀😀abcde", 2, 4, "😀😀abcd"; "when_input_contains_emojis")]
    fn get_page_returns_expected_output(source: &str, rows: usize, cols: usize, expected: &str) {
        let mut source = Box::new(BufReader::new(source.as_bytes()));
        let page = get_page(&mut source, rows, cols);

        assert_eq!(page, expected);
    }

    #[test_case("", 1, 5, ("", 1); "when_input_empty")]
    #[test_case("test", 1, 5, ("test", 1); "when_input_shorter_than_width")]
    #[test_case("testing", 1, 5, ("testi", 1); "when_input_longer_than_width")]
    #[test_case("fläder", 1, 5, ("fläde", 1); "when_input_contains_non_ascii_characters")]
    #[test_case("😀😀abcde", 1, 5, ("😀😀a", 1); "when_input_contains_emojis")]
    #[test_case("abc😀😀", 1, 4, ("abc", 1); "when_input_contains_emojis_at_the_cut_edge")]
    #[test_case("this is a test", 2, 5, ("this is a ", 2); "when_multiple_rows_requested")]
    #[test_case("abc😀a😀", 2, 4, ("abc😀a", 2); "with_multiple_rows_and_emojis_on_cut_edge")]
    fn clip_line_returns_expected_output(
        line: &str,
        rows: usize,
        width: usize,
        expected: (&str, usize),
    ) {
        let (clipped_line, clipped_rows) = clip_line(line, rows, width);

        let (expected_line, expected_rows) = expected;

        assert_eq!(clipped_rows, expected_rows);
        assert_eq!(clipped_line, expected_line);
    }
}
