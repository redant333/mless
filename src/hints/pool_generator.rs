//! Hint generation from a pool of characters.
use crate::hints::HintGenerator;

/// A [HintGenerator] that generates hints from a character pool.
///
/// It generates hints with one or two characters and attempts to
/// represent them with the least total number of characters.
/// The maximum number of hints it can provide is equal to the square
/// of hint pool size.
pub struct HintPoolGenerator {
    hint_pool: String,
}

impl HintPoolGenerator {
    /// Create a new [HintPoolGenerator] with the given character pool.
    pub fn new(hint_character_pool: &str) -> Self {
        Self {
            hint_pool: hint_character_pool.to_string(),
        }
    }
}

impl HintGenerator for HintPoolGenerator {
    fn create_hints(&self, hint_count: usize) -> Vec<String> {
        if self.hint_pool.is_empty() {
            return vec![];
        }

        let hint_pool_size = self.hint_pool.chars().count();

        if self.hint_pool.len() >= hint_count {
            return self
                .hint_pool
                .chars()
                .take(hint_count)
                .map(|char| char.to_string())
                .collect();
        }

        // In order for it to be possible to distinguish different hints
        // while the user is typing the characters, a characters that is
        // starting a hint with one character cannot be used to start a
        // hint with two characters and vice versa.
        //
        // Using a character for a one string hint allows representing
        // just one hint. Using a character to start a two character hint
        // allows representing number of hints equal to the pool size.
        //
        // Try to use every character to start a one character hint and
        // decrease the number of one character hints until the requested
        // number of hints can be represented.
        let chars_starting_one_char_hint = (0..=hint_pool_size)
            .rev()
            .find(|one_char_hints| {
                let two_char_hints = hint_pool_size - one_char_hints;
                let representable_hints = one_char_hints + two_char_hints * hint_pool_size;

                representable_hints >= hint_count
            })
            .unwrap_or(0);

        let one_char_hints = self
            .hint_pool
            .chars()
            .take(chars_starting_one_char_hint)
            .map(|char| char.to_string());

        let generate_hints_starting_with = |starting_with| {
            self.hint_pool
                .chars()
                .map(move |char| format!("{}{}", starting_with, char))
        };

        let two_char_hints = self
            .hint_pool
            .chars()
            .skip(chars_starting_one_char_hint)
            .flat_map(generate_hints_starting_with)
            .take(hint_count - chars_starting_one_char_hint);

        one_char_hints.chain(two_char_hints).collect()
    }
}

#[cfg(test)]
mod create_hints_tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", 5)]
    #[test_case("asdfgjkl", 0)]
    fn returns_empty_vector_for_empty_inputs(pool: &str, hint_count: usize) {
        let generator = HintPoolGenerator::new(pool);
        let hints = generator.create_hints(hint_count);

        assert!(hints.is_empty())
    }

    #[test_case("asdfghjkl", 5, 5, 0)] // e.g.: a s d f g
    #[test_case("asdfg", 5, 5, 0)] // e.g.: a s d f g
    #[test_case("asd", 4, 2, 2)] // e.g.: a s da ds
    #[test_case("asd", 5, 2, 3)] // e.g.: a s da ds dd
    #[test_case("asd", 6, 1, 5)] // e.g.: a sa ss sd da ds
    #[test_case("asd", 7, 1, 6)] // e.g.: a sa ss sd da ds dd
    #[test_case("asd", 8, 0, 8)] // e.g.: aa as sa ss sd da ds dd
    #[test_case("asd", 9, 0, 9)] // e.g.: aa as ad sa ss sd da ds dd
    fn returns_expected_hint_lengths(
        pool: &str,
        hint_count: usize,
        expected_one_char_hints: usize,
        expected_two_char_hints: usize,
    ) {
        let generator = HintPoolGenerator::new(pool);
        let hints = generator.create_hints(hint_count);

        assert_eq!(hints.len(), hint_count);

        let one_char_hints = hints
            .iter()
            .filter(|hint| hint.chars().count() == 1)
            .count();
        assert_eq!(one_char_hints, expected_one_char_hints);

        let two_char_hints = hints
            .iter()
            .filter(|hint| hint.chars().count() == 2)
            .count();
        assert_eq!(two_char_hints, expected_two_char_hints);

        for hint in hints {
            let all_hint_chars_in_pool = hint.chars().all(|char| pool.contains(char));
            assert!(all_hint_chars_in_pool);
        }
    }

    #[test]
    fn returns_fewer_hints_if_not_all_can_be_represented() {
        let pool = "asd";
        let max_hints = pool.chars().count() * pool.chars().count();

        let generator = HintPoolGenerator::new(pool);
        let hints = generator.create_hints(999);

        assert_eq!(hints.len(), max_hints);
    }
}
