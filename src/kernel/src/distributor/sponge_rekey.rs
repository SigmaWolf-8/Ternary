// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Per-frame TLSponge-385 state advance for framebuffer encryption.
// Each frame gets a unique keystream derived from advancing the sponge.

use alloc::vec::Vec;

pub const SPONGE_WIDTH_TRITS: usize = 729;
pub const SPONGE_SECURITY_BITS: usize = 385;
pub const SPONGE_ROUNDS: usize = 9;

pub const KEYSTREAM_BYTES_1080P: usize = 1920 * 1080 * 4;
pub const KEYSTREAM_BYTES_4K: usize = 3840 * 2160 * 4;

#[derive(Debug, Clone)]
pub struct SpongeRekeyState {
    frame_counter: u64,
    state: Vec<u8>,
    keystream_len: usize,
}

impl SpongeRekeyState {
    pub fn new(initial_key: &[u8], resolution: FrameResolution) -> Self {
        let mut state = Vec::with_capacity(SPONGE_WIDTH_TRITS);
        for (i, &b) in initial_key.iter().enumerate() {
            if i >= SPONGE_WIDTH_TRITS {
                break;
            }
            state.push(b);
        }
        while state.len() < SPONGE_WIDTH_TRITS {
            state.push(0);
        }

        let keystream_len = resolution.pixel_bytes();

        Self {
            frame_counter: 0,
            state,
            keystream_len,
        }
    }

    pub fn advance_frame(&mut self) -> Vec<u8> {
        self.frame_counter += 1;

        let fc_bytes = self.frame_counter.to_le_bytes();
        for (i, &b) in fc_bytes.iter().enumerate() {
            if i < self.state.len() {
                self.state[i] ^= b;
            }
        }

        for round in 0..SPONGE_ROUNDS {
            self.permute_round(round);
        }

        let rate = SPONGE_WIDTH_TRITS / 2;
        let mut keystream = Vec::with_capacity(self.keystream_len);

        while keystream.len() < self.keystream_len {
            keystream.extend_from_slice(&self.state[..rate.min(self.state.len())]);

            for round in 0..SPONGE_ROUNDS {
                self.permute_round(round);
            }
        }

        keystream.truncate(self.keystream_len);
        keystream
    }

    fn permute_round(&mut self, round: usize) {
        let len = self.state.len();
        if len < 3 {
            return;
        }

        let mut scratch = Vec::with_capacity(len);
        for i in 0..len {
            let a = self.state[i] as u16;
            let b = self.state[(i + 1) % len] as u16;
            let c = self.state[(i + 2) % len] as u16;
            let mixed = (a ^ (!b & c)) as u8;
            scratch.push(mixed);
        }
        self.state.copy_from_slice(&scratch);

        let round_const = (round as u8).wrapping_mul(0x9E).wrapping_add(0x37);
        for (i, byte) in self.state.iter_mut().enumerate() {
            *byte ^= round_const.wrapping_add(i as u8);
        }

        let rot = (round * 7 + 3) % len;
        self.state.rotate_left(rot);
    }

    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }

    pub fn xor_framebuffer(&self, framebuffer: &mut [u8]) {
        let key = &self.state;
        let key_len = key.len();
        if key_len == 0 {
            return;
        }
        for (i, pixel) in framebuffer.iter_mut().enumerate() {
            *pixel ^= key[i % key_len];
        }
    }

    pub fn reset(&mut self, new_key: &[u8]) {
        self.frame_counter = 0;
        for (i, byte) in self.state.iter_mut().enumerate() {
            *byte = if i < new_key.len() { new_key[i] } else { 0 };
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameResolution {
    Hd1080,
    Uhd4K,
    Custom { width: u32, height: u32 },
}

impl FrameResolution {
    pub fn pixel_bytes(&self) -> usize {
        match self {
            FrameResolution::Hd1080 => KEYSTREAM_BYTES_1080P,
            FrameResolution::Uhd4K => KEYSTREAM_BYTES_4K,
            FrameResolution::Custom { width, height } => {
                (*width as usize) * (*height as usize) * 4
            }
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            FrameResolution::Hd1080 => (1920, 1080),
            FrameResolution::Uhd4K => (3840, 2160),
            FrameResolution::Custom { width, height } => (*width, *height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sponge_constants() {
        assert_eq!(SPONGE_WIDTH_TRITS, 729);
        assert_eq!(729u32, 3u32.pow(6));
        assert_eq!(SPONGE_SECURITY_BITS, 385);
        assert_eq!(SPONGE_ROUNDS, 9);
    }

    #[test]
    fn test_sponge_rekey_advances() {
        let key = [0x42u8; 32];
        let mut state = SpongeRekeyState::new(&key, FrameResolution::Custom { width: 8, height: 8 });
        assert_eq!(state.frame_counter(), 0);

        let ks1 = state.advance_frame();
        assert_eq!(state.frame_counter(), 1);
        assert_eq!(ks1.len(), 8 * 8 * 4);

        let ks2 = state.advance_frame();
        assert_eq!(state.frame_counter(), 2);
        assert_eq!(ks2.len(), 8 * 8 * 4);

        assert_ne!(ks1, ks2, "each frame must produce a different keystream");
    }

    #[test]
    fn test_keystream_full_resolution() {
        let key = [0x01u8; 32];
        let mut state = SpongeRekeyState::new(&key, FrameResolution::Custom { width: 100, height: 100 });
        let ks = state.advance_frame();
        assert_eq!(ks.len(), 100 * 100 * 4, "keystream must match full resolution");
    }

    #[test]
    fn test_xor_framebuffer_roundtrip() {
        let key = [0xABu8; 16];
        let state = SpongeRekeyState::new(&key, FrameResolution::Custom { width: 4, height: 4 });

        let original = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut buf = original.clone();

        state.xor_framebuffer(&mut buf);
        assert_ne!(buf, original);

        state.xor_framebuffer(&mut buf);
        assert_eq!(buf, original);
    }

    #[test]
    fn test_frame_resolution() {
        assert_eq!(FrameResolution::Hd1080.pixel_bytes(), 1920 * 1080 * 4);
        assert_eq!(FrameResolution::Uhd4K.dimensions(), (3840, 2160));
    }
}
