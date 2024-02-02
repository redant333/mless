use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::configuration::Config;

pub struct InputHandler {}

pub enum Action {
    Exit,
}

const KEYBINDING_EXIT: char = 'q';

impl InputHandler {
    pub fn from_config(_config: &Config) -> InputHandler {
        InputHandler {}
    }

    pub fn get_action(&self, event: Event) -> Option<Action> {
        match event {
            Event::Key(key) => self.get_key_action(key),
            _ => None,
        }
    }

    fn get_key_action(&self, key: KeyEvent) -> Option<Action> {
        match key {
            KeyEvent {
                code: KeyCode::Char(KEYBINDING_EXIT),
                ..
            } => Some(Action::Exit),
            _ => None,
        }
    }
}
