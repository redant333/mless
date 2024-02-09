use crate::{input_handler::KeyPress, renderer::Draw};

use super::{Mode, ModeAction};

pub struct RegexMode {}

impl Mode for RegexMode {
    fn handle_key_press(&mut self, _key: KeyPress) -> Option<ModeAction> {
        None
    }

    fn get_draw_instructions(&self) -> Vec<Draw> {
        vec![Draw::Data]
    }
}
