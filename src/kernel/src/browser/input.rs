// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Browser input handler — receives TIS-27 encoded keycodes,
// decodes at DOM handler level (last possible moment).
// Direct kernel call from input/keyboard.rs — no IPC, no shared memory buffer.
//
// Flow: Serial receive (COM1/PL011/SBI) at interrupt level →
//       TIS-27 encode before any buffer →
//       direct kernel call to BrowserInputHandler →
//       decode to Unicode inside Boa's DOM event handler

use crate::input::keyboard::{EncodedKey, Tis27State};
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_EVENT_QUEUE_SIZE: usize = 256;

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
    pub decoded_char: Option<char>,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub timestamp_ns: u64,
    pub target_node_id: Option<u32>,
}

impl InputEvent {
    pub fn is_keyboard(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::KeyDown | InputEventType::KeyUp | InputEventType::KeyPress
        )
    }

    pub fn is_mouse(&self) -> bool {
        matches!(
            self.event_type,
            InputEventType::MouseMove | InputEventType::MouseDown | InputEventType::MouseUp | InputEventType::Scroll
        )
    }
}

pub fn scancode_to_unicode(scancode: u8) -> Option<char> {
    match scancode {
        0x02..=0x0B => {
            let digit = if scancode == 0x0B { b'0' } else { b'0' + scancode - 1 };
            Some(digit as char)
        }
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x39 => Some(' '),
        0x1C => Some('\n'),
        0x0E => Some('\x08'),
        0x0F => Some('\t'),
        _ => None,
    }
}

pub struct BrowserInputHandler {
    decoder: Tis27State,
    event_queue: Vec<InputEvent>,
    active_tab_id: Option<u32>,
    events_processed: u64,
    events_dropped: u64,
}

impl BrowserInputHandler {
    pub fn new(session_key: [u8; 32]) -> Self {
        Self {
            decoder: Tis27State::new(session_key),
            event_queue: Vec::new(),
            active_tab_id: None,
            events_processed: 0,
            events_dropped: 0,
        }
    }

    pub fn set_active_tab(&mut self, tab_id: u32) {
        self.active_tab_id = Some(tab_id);
    }

    pub fn receive_key(&mut self, encoded: EncodedKey, event_type: InputEventType) {
        if self.event_queue.len() >= MAX_EVENT_QUEUE_SIZE {
            self.events_dropped += 1;
            return;
        }

        self.event_queue.push(InputEvent {
            event_type,
            encoded_key: Some(encoded),
            decoded_char: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            timestamp_ns: 0,
            target_node_id: None,
        });
        self.events_processed += 1;
    }

    pub fn receive_key_with_timestamp(&mut self, encoded: EncodedKey, event_type: InputEventType, timestamp_ns: u64) {
        if self.event_queue.len() >= MAX_EVENT_QUEUE_SIZE {
            self.events_dropped += 1;
            return;
        }

        self.event_queue.push(InputEvent {
            event_type,
            encoded_key: Some(encoded),
            decoded_char: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            timestamp_ns,
            target_node_id: None,
        });
        self.events_processed += 1;
    }

    pub fn receive_mouse(&mut self, x: f32, y: f32, event_type: InputEventType) {
        if self.event_queue.len() >= MAX_EVENT_QUEUE_SIZE {
            self.events_dropped += 1;
            return;
        }

        self.event_queue.push(InputEvent {
            event_type,
            encoded_key: None,
            decoded_char: None,
            mouse_x: x,
            mouse_y: y,
            timestamp_ns: 0,
            target_node_id: None,
        });
        self.events_processed += 1;
    }

    pub fn receive_mouse_with_target(&mut self, x: f32, y: f32, event_type: InputEventType, target_node_id: u32) {
        if self.event_queue.len() >= MAX_EVENT_QUEUE_SIZE {
            self.events_dropped += 1;
            return;
        }

        self.event_queue.push(InputEvent {
            event_type,
            encoded_key: None,
            decoded_char: None,
            mouse_x: x,
            mouse_y: y,
            timestamp_ns: 0,
            target_node_id: Some(target_node_id),
        });
        self.events_processed += 1;
    }

    pub fn decode_key(&mut self, encoded: &EncodedKey) -> Option<u8> {
        self.decoder.decode(encoded)
    }

    pub fn decode_key_to_unicode(&mut self, encoded: &EncodedKey) -> Option<char> {
        self.decoder.decode(encoded).and_then(scancode_to_unicode)
    }

    pub fn process_pending_events(&mut self) -> Vec<InputEvent> {
        let events: Vec<InputEvent> = core::mem::take(&mut self.event_queue);
        let mut processed = Vec::with_capacity(events.len());

        for mut event in events {
            if let Some(ref encoded) = event.encoded_key {
                let encoded_copy = *encoded;
                event.decoded_char = self.decode_key_to_unicode(&encoded_copy);
            }
            processed.push(event);
        }

        processed
    }

    pub fn drain_events(&mut self) -> Vec<InputEvent> {
        core::mem::take(&mut self.event_queue)
    }

    pub fn pending_events(&self) -> usize {
        self.event_queue.len()
    }

    pub fn events_processed(&self) -> u64 {
        self.events_processed
    }

    pub fn events_dropped(&self) -> u64 {
        self.events_dropped
    }

    pub fn active_tab_id(&self) -> Option<u32> {
        self.active_tab_id
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

    #[test]
    fn test_scancode_to_unicode() {
        assert_eq!(scancode_to_unicode(0x1E), Some('a'));
        assert_eq!(scancode_to_unicode(0x39), Some(' '));
        assert_eq!(scancode_to_unicode(0x1C), Some('\n'));
        assert_eq!(scancode_to_unicode(0x02), Some('1'));
        assert_eq!(scancode_to_unicode(0xFF), None);
    }

    #[test]
    fn test_event_queue_overflow() {
        let key = [0u8; 32];
        let mut handler = BrowserInputHandler::new(key);
        let mut kb = KeyboardHandler::new(key);

        for _ in 0..MAX_EVENT_QUEUE_SIZE + 10 {
            let encoded = kb.handle_key_event(0x1E);
            handler.receive_key(encoded, InputEventType::KeyDown);
        }

        assert_eq!(handler.pending_events(), MAX_EVENT_QUEUE_SIZE);
        assert!(handler.events_dropped() > 0);
    }

    #[test]
    fn test_active_tab_tracking() {
        let key = [0u8; 32];
        let mut handler = BrowserInputHandler::new(key);
        assert_eq!(handler.active_tab_id(), None);
        handler.set_active_tab(42);
        assert_eq!(handler.active_tab_id(), Some(42));
    }

    #[test]
    fn test_event_classification() {
        let kbd_event = InputEvent {
            event_type: InputEventType::KeyDown,
            encoded_key: None,
            decoded_char: Some('a'),
            mouse_x: 0.0,
            mouse_y: 0.0,
            timestamp_ns: 0,
            target_node_id: None,
        };
        assert!(kbd_event.is_keyboard());
        assert!(!kbd_event.is_mouse());

        let mouse_event = InputEvent {
            event_type: InputEventType::MouseDown,
            encoded_key: None,
            decoded_char: None,
            mouse_x: 100.0,
            mouse_y: 200.0,
            timestamp_ns: 0,
            target_node_id: Some(5),
        };
        assert!(!mouse_event.is_keyboard());
        assert!(mouse_event.is_mouse());
    }

    #[test]
    fn test_mouse_with_target() {
        let key = [0u8; 32];
        let mut handler = BrowserInputHandler::new(key);
        handler.receive_mouse_with_target(50.0, 100.0, InputEventType::MouseDown, 7);
        let events = handler.drain_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target_node_id, Some(7));
    }

    #[test]
    fn test_events_processed_counter() {
        let key = [0u8; 32];
        let mut handler = BrowserInputHandler::new(key);
        handler.receive_mouse(10.0, 20.0, InputEventType::MouseMove);
        handler.receive_mouse(30.0, 40.0, InputEventType::MouseMove);
        assert_eq!(handler.events_processed(), 2);
    }
}
