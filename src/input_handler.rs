//! Handling of input events before they are delivered to the current mode.
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::configuration::Config;

/// Handle the input from the user.
///
/// The two main reasons for the existence of this struct are:
/// 1. To provide a layer of input handling that does not depend on the current mode.
/// 2. To translate [crossterm] specific events into the the format used in this application.
pub struct InputHandler {}

/// Representation of a key press that is delivered to the rest of the application.
#[derive(Debug)]
pub struct KeyPress {
    /// The key that was pressed.
    ///
    /// Note that this is currently only convenient for representing keys
    /// associated with a specific character (e.g. `'a'`, `'A'`). It will need
    /// to be modified if keys with modifiers (`Ctrl`, `Alt`) or keys that don't have
    /// a good one-character representation (`F1`, `Backspace`) need to be supported.
    pub key: char,
}

/// The action that resulted from the input.
#[derive(Debug)]
pub enum Action {
    /// Exit the application without providing any selected text.
    Exit,
    /// Forward the given [KeyPress] to the active [crate::modes::Mode].
    ForwardKeyPress(KeyPress),
    /// Terminal changed size
    Resize,
    /// Go to a state where the user can choose to switch the mode
    GoToModeSelection,
}

impl InputHandler {
    /// Create an [InputHandler] by using the relevant parts of the given config.
    pub fn from_config(_config: &Config) -> InputHandler {
        InputHandler {}
    }

    /// Get the [Action] (if any) resulting from the given input event.
    pub fn get_action(&self, event: Event) -> Option<Action> {
        match event {
            Event::Resize(_, _) => Some(Action::Resize),
            Event::Key(key) => self.get_key_action(key),
            _ => None,
        }
    }

    /// Get the [Action] (if any) resulting from the given key press.
    ///
    /// This handles specifically key actions and not mouse actions, window
    /// resize or similar.
    fn get_key_action(&self, key: KeyEvent) -> Option<Action> {
        match key {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => Some(Action::Exit),
            KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => Some(Action::GoToModeSelection),
            KeyEvent {
                code: KeyCode::Char(key),
                ..
            } => Some(Action::ForwardKeyPress(KeyPress { key })),
            _ => None,
        }
    }
}
