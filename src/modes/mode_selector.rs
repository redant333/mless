use crossterm::style::Color;

use crate::{
    configuration,
    rendering::{DataOverlay, DrawInstruction, StyledSegment, TextStyle},
};

use super::Mode;

/// A mode that allows the user to change to a different selection mode.
pub struct ModeSelectorMode<'a> {
    modes: &'a [configuration::Mode],
}

impl<'a> ModeSelectorMode<'a> {
    pub fn new(modes: &'a [configuration::Mode]) -> Self {
        Self { modes }
    }
}

impl<'a> Mode for ModeSelectorMode<'a> {
    fn handle_key_press(
        &mut self,
        _key: crate::input_handler::KeyPress,
    ) -> Option<super::ModeEvent> {
        None
    }

    fn get_draw_instructions(&self) -> Vec<DrawInstruction> {
        let heading = std::iter::once("MODE SELECTION |".to_string());
        let modes = self
            .modes
            .iter()
            .map(|mode| format!(" [{}] {}", mode.hotkey, mode.name));
        let text: String = heading.chain(modes).collect();

        vec![DrawInstruction::StyledData {
            styled_segments: vec![StyledSegment {
                start: 0,
                length: text.len(),
                style: TextStyle {
                    foreground: Color::Black,
                    background: Color::Yellow,
                },
            }],
            text_overlays: vec![DataOverlay { text, location: 0 }],
        }]
    }
}
