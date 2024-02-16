use crossterm::style::Color;

use crate::{
    input_handler::KeyPress,
    renderer::{Draw, TextStyle},
};

use super::{Mode, ModeAction};

pub struct RegexMode {}

impl Mode for RegexMode {
    fn handle_key_press(&mut self, _key: KeyPress) -> Option<ModeAction> {
        None
    }

    fn get_draw_instructions(&self) -> Vec<Draw> {
        vec![
            Draw::Data,
            Draw::TextRelativeToData {
                text: "test".into(),
                location: 0,
                style: TextStyle {
                    foreground: Color::Red,
                    background: Color::Yellow,
                },
            },
        ]
    }
}
