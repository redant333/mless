use crate::{configuration, rendering::DrawInstruction};

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
        let modes = self
            .modes
            .iter()
            .map(|mode| (mode.hotkey, mode.name.clone()))
            .collect();

        vec![
            DrawInstruction::Data,
            DrawInstruction::ModeSelectionDialog(modes),
        ]
    }
}
