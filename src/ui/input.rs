//! Input Handling Module

pub struct InputHandler;

impl InputHandler {
    pub fn handle_keypress(key_code: u32) -> Option<InputAction> {
        match key_code {
            0x1B => Some(InputAction::Escape),
            0x70 => Some(InputAction::F1),
            0x71 => Some(InputAction::F2),
            0x72 => Some(InputAction::F3),
            0x73 => Some(InputAction::F4),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputAction {
    Escape,
    F1,
    F2,
    F3,
    F4,
    EmergencyUnlock,
}
