use crate::{input_handler::KeyPress, renderer::Draw};

mod regex;
pub use regex::RegexMode;

pub trait Mode {
    fn handle_key_press(&mut self, key: KeyPress) -> Option<ModeEvent>;
    fn get_draw_instructions(&self) -> Vec<Draw>;
}

pub enum ModeEvent {
    TextSelected(String),
}
