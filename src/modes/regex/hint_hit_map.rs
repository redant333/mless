//! Structs for handling mapping between hints and hits.
use std::collections::{HashMap, HashSet};

use log::{info, trace};

use crate::hints::HintGenerator;

#[derive(Debug)]
/// Struct that records a hit(match) that can be selected.
pub struct Hit {
    /// Byte offset of the start of the hit.
    ///
    /// This is represented as character offset from the first character.
    pub start: usize,

    /// Length of the hit in bytes.
    pub length: usize,

    /// The text of the hit.
    ///
    /// This will be returned to the user if this hit is selected.
    pub text: String,
}

#[derive(Debug)]
/// Struct used to keep assosiations between hints and the hits they are assigned to.
/// Allows one hint to be associated with multiple hits. This is needed to make it
/// possible for multiple hits with the same text but on different locations to be
/// assigned to the same hint.
pub struct HintHitMap {
    pub pairs: Vec<(String, Hit)>,
}

impl HintHitMap {
    /// Create a mapping of hints to hits from the given collection of hits and the generator.
    pub fn new(hits: Vec<Hit>, hint_generator: &dyn HintGenerator) -> Self {
        let unique_hit_count = hits
            .iter()
            .map(|hit| hit.text.clone())
            .collect::<HashSet<String>>()
            .len();
        info!("Number of unique hits {unique_hit_count}");
        let hints = hint_generator.create_hints(unique_hit_count);
        let mut hint_iter = hints.iter();

        let mut hit_hint_map = HashMap::<String, String>::new();
        let mut pairs: Vec<(String, Hit)> = vec![];

        for hit in hits.into_iter() {
            let hint = if hit_hint_map.contains_key(&hit.text) {
                trace!("Hit <{}> already in hit_hint_map", hit.text);
                hit_hint_map[&hit.text].clone()
            } else if let Some(hint) = hint_iter.next() {
                trace!("Using new hint {} for hit <{}>", hint, hit.text);
                hit_hint_map.insert(hit.text.clone(), hint.clone());
                hint.clone()
            } else {
                info!("Not enough hints for all the hits, giving up");
                break;
            };

            pairs.push((hint, hit));
        }

        Self { pairs }
    }

    /// Get the first [Hit] associated with the given hint string.
    pub fn get_hit(&self, hint: &str) -> Option<&Hit> {
        self.pairs //
            .iter()
            .find_map(
                |(pair_hint, hit)| {
                    if pair_hint == hint {
                        Some(hit)
                    } else {
                        None
                    }
                },
            )
    }

    /// Check if the map contains a hint beginning with the given prefix.
    /// The [Hit] value does not affect the outcome of this function.
    pub fn has_hint_with_prefix(&self, prefix: &str) -> bool {
        self.pairs
            .iter()
            .any(|(pair_hint, _)| pair_hint.starts_with(prefix))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test]
    fn get_hit_returns_some_hit_when_exists() {
        let hint_hit_map = HintHitMap {
            #[rustfmt::skip]
            pairs: vec![
                ("a".to_string(), Hit {start: 0, length: 5, text: "stuff".to_string() }),
                ("b".to_string(), Hit {start: 5, length: 6, text: "things".to_string() }),
                ("c".to_string(), Hit {start: 9, length: 7, text: "fidgets".to_string() }),
            ],
        };

        let hit = hint_hit_map.get_hit("b").unwrap();
        assert_eq!(hit.text, hint_hit_map.pairs[1].1.text);
    }

    #[test]
    fn get_hit_returns_none_when_hit_does_not_exist() {
        let hint_hit_map = HintHitMap {
            #[rustfmt::skip]
            pairs: vec![
                ("a".to_string(), Hit {start: 0, length: 5, text: "stuff".to_string() }),
                ("b".to_string(), Hit {start: 5, length: 6, text: "things".to_string() }),
                ("c".to_string(), Hit {start: 9, length: 7, text: "fidgets".to_string() }),
            ],
        };

        let hit = hint_hit_map.get_hit("x");
        assert!(hit.is_none());
    }

    #[test_case("a", true)]
    #[test_case("x", false)]
    fn has_hint_with_prefix_returns_expected_value(prefix: &str, expected: bool) {
        let hint_hit_map = HintHitMap {
            #[rustfmt::skip]
            pairs: vec![
                ("aa".to_string(), Hit {start: 0, length: 5, text: "stuff".to_string() }),
                ("ab".to_string(), Hit {start: 5, length: 6, text: "things".to_string() }),
                ("ac".to_string(), Hit {start: 9, length: 7, text: "fidgets".to_string() }),
            ],
        };

        let has = hint_hit_map.has_hint_with_prefix(prefix);
        assert_eq!(has, expected);
    }
}
