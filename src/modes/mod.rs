//! Different selection modes.
use crate::{input_handler::KeyPress, rendering::Draw};

mod regex;
pub use regex::RegexMode;

/// The trait that defines all selection modes.
pub trait Mode {
    /// Handle the key press from the user.
    ///
    /// This can result in a [ModeEvent] and modifies the internal
    /// state of the mode.
    fn handle_key_press(&mut self, key: KeyPress) -> Option<ModeEvent>;

    /// Specify the draw instructions for [crate::rendering::Renderer].
    ///
    /// Note that the renderer does not display anything if the returned
    /// vector is empty.
    fn get_draw_instructions(&self) -> Vec<Draw>;
}

/// Enum that specifies the events happening inside the mode.
#[derive(Debug)]
pub enum ModeEvent {
    /// The test selection has finished and resulted in the given string.
    TextSelected(String),
}
