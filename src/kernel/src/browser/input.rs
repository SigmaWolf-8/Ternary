// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Browser input handler — receives TIS-27 encoded keycodes,
// decodes at DOM handler level (last possible moment).
// Direct kernel call from input/keyboard.rs.

use crate::input::keyboard::{EncodedKey, Tis27State};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    KeyDown,
    KeyUp,
    KeyPress,
    MouseMove,
    MouseDown,
    MouseUp,
    Scroll,
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub encoded_key: Option<EncodedKey>,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub timestamp_ns: u64,
}

pub struct BrowserInputHandler {
    decoder: Tis27State,
    event_queue: Vec<InputEvent>,
}

impl BrowserInputHandler {
    pub fn new(session_key: [u8; 32]) -> Self {
        Self {
            decoder: Tis27State::new(session_key),
            event_queue: Vec::new(),
        }
    }

    pub fn receive_key(&mut self, encoded: EncodedKey, event_type: InputEventType) {
        self.event_queue.push(InputEvent {
            event_type,
            encoded_key: Some(encoded),
            mouse_x: 0.0,
            mouse_y: 0.0,
            timestamp_ns: 0,
        });
    }

    pub fn receive_mouse(&mut self, x: f32, y: f32, event_type: InputEventType) {
        self.event_queue.push(InputEvent {
            event_type,
            encoded_key: None,
            mouse_x: x,
            mouse_y: y,
            timestamp_ns: 0,
        });
    }

    pub fn decode_key(&mut self, encoded: &EncodedKey) -> Option<u8> {
        self.decoder.decode(encoded)
    }

    pub fn drain_events(&mut self) -> Vec<InputEvent> {
        core::mem::take(&mut self.event_queue)
    }

    pub fn pending_events(&self) -> usize {
        self.event_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::keyboard::KeyboardHandler;

    #[test]
    fn test_input_handler_key_events() {
        let key = [0x42u8; 32];
        let mut handler = BrowserInputHandler::new(key);
        let mut kb = KeyboardHandler::new(key);

        let encoded = kb.handle_key_event(0x41);
        handler.receive_key(encoded, InputEventType::KeyDown);

        assert_eq!(handler.pending_events(), 1);

        let events = handler.drain_events();
        assert_eq!(events.len(), 1);
        assert_eq!(handler.pending_events(), 0);
    }

    #[test]
    fn test_input_handler_mouse() {
        let key = [0u8; 32];
        let mut handler = BrowserInputHandler::new(key);
        handler.receive_mouse(100.0, 200.0, InputEventType::MouseMove);
        assert_eq!(handler.pending_events(), 1);
    }
}
