use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct KeyConfig {
    pub exit_key: KeyEvent,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            exit_key: KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
            },
        }
    }
}
