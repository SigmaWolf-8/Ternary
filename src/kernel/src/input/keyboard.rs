// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// TIS-27 Keyboard Encoding — encode scancodes BEFORE any buffer.
// 54-trit sponge, 4 rounds, 43-bit integrity.
// Decoded to Unicode inside Boa's DOM event handler at last possible moment.

pub const TIS27_TRITS: usize = 54;
pub const TIS27_ROUNDS: usize = 4;
pub const TIS27_SECURITY_BITS: usize = 43;

#[derive(Debug, Clone)]
pub struct Tis27State {
    state: [u8; TIS27_TRITS],
    session_key: [u8; 32],
}

impl Tis27State {
    pub fn new(session_key: [u8; 32]) -> Self {
        let mut state = [0u8; TIS27_TRITS];
        for (i, &b) in session_key.iter().enumerate() {
            if i < TIS27_TRITS {
                state[i] = b % 3;
            }
        }
        Self { state, session_key }
    }

    pub fn encode(&mut self, scancode: u8) -> EncodedKey {
        self.absorb_scancode(scancode);
        self.permute();
        let tag = self.squeeze_tag();

        EncodedKey {
            encoded_scancode: scancode ^ self.state[0],
            integrity_tag: tag,
        }
    }

    pub fn decode(&mut self, encoded: &EncodedKey) -> Option<u8> {
        let scancode = encoded.encoded_scancode ^ self.state[0];
        self.absorb_scancode(scancode);
        self.permute();
        let expected_tag = self.squeeze_tag();

        if expected_tag == encoded.integrity_tag {
            Some(scancode)
        } else {
            None
        }
    }

    fn absorb_scancode(&mut self, scancode: u8) {
        let trits = byte_to_trits(scancode);
        for i in 0..6.min(TIS27_TRITS) {
            self.state[i] = (self.state[i] + trits[i]) % 3;
        }
    }

    fn permute(&mut self) {
        for round in 0..TIS27_ROUNDS {
            for i in 0..TIS27_TRITS {
                let a = self.state[i];
                let b = self.state[(i + 1) % TIS27_TRITS];
                let c = self.state[(i + 2) % TIS27_TRITS];
                self.state[i] = (a + b * 2 + c + round as u8) % 3;
            }

            let rot = (round * 7 + 1) % TIS27_TRITS;
            self.state.rotate_left(rot);
        }
    }

    fn squeeze_tag(&self) -> u64 {
        let mut tag: u64 = 0;
        for i in 0..27.min(TIS27_TRITS) {
            tag = tag.wrapping_mul(3).wrapping_add(self.state[i] as u64);
        }
        tag
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EncodedKey {
    pub encoded_scancode: u8,
    pub integrity_tag: u64,
}

fn byte_to_trits(b: u8) -> [u8; 6] {
    let mut trits = [0u8; 6];
    let mut val = b;
    for trit in trits.iter_mut() {
        *trit = val % 3;
        val /= 3;
    }
    trits
}

pub struct KeyboardHandler {
    encoder: Tis27State,
}

impl KeyboardHandler {
    pub fn new(session_key: [u8; 32]) -> Self {
        Self {
            encoder: Tis27State::new(session_key),
        }
    }

    pub fn handle_key_event(&mut self, scancode: u8) -> EncodedKey {
        self.encoder.encode(scancode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tis27_constants() {
        assert_eq!(TIS27_TRITS, 54);
        assert_eq!(TIS27_ROUNDS, 4);
        assert_eq!(TIS27_SECURITY_BITS, 43);
    }

    #[test]
    fn test_byte_to_trits() {
        let trits = byte_to_trits(0);
        assert_eq!(trits, [0, 0, 0, 0, 0, 0]);

        let trits = byte_to_trits(1);
        assert_eq!(trits, [1, 0, 0, 0, 0, 0]);

        let trits = byte_to_trits(3);
        assert_eq!(trits, [0, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn test_encode_produces_different_outputs() {
        let key = [0x42u8; 32];
        let mut state = Tis27State::new(key);
        let e1 = state.encode(0x1E);
        let mut state2 = Tis27State::new(key);
        let e2 = state2.encode(0x1F);
        assert_ne!(e1.encoded_scancode, e2.encoded_scancode);
    }

    #[test]
    fn test_keyboard_handler() {
        let key = [0x55u8; 32];
        let mut handler = KeyboardHandler::new(key);
        let encoded = handler.handle_key_event(0x41);
        assert_ne!(encoded.encoded_scancode, 0x41);
    }
}
