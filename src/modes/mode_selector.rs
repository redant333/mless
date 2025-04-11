use crossterm::style::Color;

use crate::rendering::{DataOverlay, DrawInstruction, StyledSegment, TextStyle};

use super::Mode;

/// A mode that allows the user to change to a different selection mode.
pub struct ModeSelectorMode {}

impl Mode for ModeSelectorMode {
    fn handle_key_press(
        &mut self,
        _key: crate::input_handler::KeyPress,
    ) -> Option<super::ModeEvent> {
        None
    }

    fn get_draw_instructions(&self) -> Vec<DrawInstruction> {
        vec![DrawInstruction::StyledData {
            styled_segments: vec![StyledSegment {
                start: 0,
                length: 14,
                style: TextStyle {
                    foreground: Color::Black,
                    background: Color::Yellow,
                },
            }],
            text_overlays: vec![DataOverlay {
                text: "MODE SELECTION".to_string(),
                location: 0,
            }],
        }]
    }
}
