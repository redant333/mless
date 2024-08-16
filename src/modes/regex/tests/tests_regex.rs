use crate::{configuration::RegexArgs, hints::MockHintGenerator};
use test_case::test_case;

use super::*;

fn has_overlay_at_location(overlays: &[DataOverlay], location: usize) -> bool {
    overlays.iter().any(|overlay| overlay.location == location)
}

fn has_highlight(highlights: &[StyledSegment], start: usize, length: usize) -> bool {
    highlights
        .iter()
        .any(|highlight| highlight.start == start && highlight.length == length)
}

fn get_draw_instructions(
    text: &str,
    regexes: Vec<String>,
    hints: Vec<String>,
) -> (Vec<DataOverlay>, Vec<StyledSegment>) {
    let args = RegexArgs { regexes };

    let mut hint_generator = Box::new(MockHintGenerator::new());
    hint_generator.expect_create_hints().return_const(hints);

    let mode = RegexMode::new(text, &args, hint_generator);
    let Draw::StyledData {
        text_overlays,
        styled_segments,
    } = mode.get_draw_instructions().into_iter().next().unwrap();

    (text_overlays, styled_segments)
}

#[test]
fn produces_expected_highlights_and_overlays_for_simple_text() {
    let (text_overlays, styled_segments) = get_draw_instructions(
        "things and stuff",
        vec![r"[a-z]{4,}".into()],
        vec!["a".into(), "b".into()],
    );

    assert_eq!(text_overlays.len(), 2);
    assert!(has_overlay_at_location(&text_overlays, 0));
    assert!(has_overlay_at_location(&text_overlays, 11));

    assert_eq!(styled_segments.len(), 4);
    // Highlights for "things" match
    assert!(has_highlight(&styled_segments, 0, 6));
    assert!(has_highlight(&styled_segments, 0, 1));

    // Highlights for "stuff" match
    assert!(has_highlight(&styled_segments, 11, 5));
    assert!(has_highlight(&styled_segments, 11, 1));
}

#[test]
fn produces_expected_highlights_and_overlays_for_colored_text() {
    let color_1 = "\x1b[0;31m";
    let color_2 = "\x1b[0;32m";
    let color_reset = "\x1b[0m";

    let (text_overlays, styled_segments) = get_draw_instructions(
            &format!("{color_1}things{color_reset} and {color_2}stuff{color_reset} and {color_1}mice{color_reset} and {color_2}mud{color_reset}"),
            vec![r"[a-z]{4,}".into()],
            vec!["a".into(), "b".into(), "c".into()],
        );

    println!("{:#?}", text_overlays);
    println!("{:#?}", styled_segments);

    assert_eq!(text_overlays.len(), 3);
    assert!(has_overlay_at_location(&text_overlays, 7));
    assert!(has_overlay_at_location(&text_overlays, 29));
    assert!(has_overlay_at_location(&text_overlays, 50));

    assert_eq!(styled_segments.len(), 6);
    // Highlights for "things" match
    assert!(has_highlight(&styled_segments, 7, 6));
    assert!(has_highlight(&styled_segments, 7, 1));

    // Highlights for "stuff" match
    assert!(has_highlight(&styled_segments, 29, 5));
    assert!(has_highlight(&styled_segments, 29, 1));

    // Highlights for "mice" match
    assert!(has_highlight(&styled_segments, 50, 4));
    assert!(has_highlight(&styled_segments, 50, 1));
}

#[test_case(&[(2,4), (6, 8)], 0, 0)]
#[test_case(&[(2,4), (6, 8)], 1, 1)]
#[test_case(&[(2,4), (6, 8)], 2, 4)]
#[test_case(&[(2,4), (6, 8)], 3, 5)]
#[test_case(&[(2,4), (6, 8)], 4, 8)]
#[test_case(&[], 4, 4)]
fn get_original_index_returns_correct_value(
    removed_ranges: &[(usize, usize)],
    index: usize,
    expected: usize,
) {
    assert_eq!(get_original_index(removed_ranges, index), expected);
}
