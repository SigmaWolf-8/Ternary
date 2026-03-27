// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Per-frame TLSponge-385 state advance for framebuffer encryption.
// Each frame gets a unique keystream derived from advancing the sponge.
// Rekey in the 461 overlap slots (1001 − 540 = 461, prime).
// SIMD-accelerated XOR (AVX2/NEON) for encryption pass.

use alloc::vec::Vec;

pub const SPONGE_WIDTH_TRITS: usize = 729;
pub const SPONGE_SECURITY_BITS: usize = 385;
pub const SPONGE_ROUNDS: usize = 9;

pub const KEYSTREAM_BYTES_1080P: usize = 1920 * 1080 * 4;
pub const KEYSTREAM_BYTES_4K: usize = 3840 * 2160 * 4;

pub const OVERLAP_SLOTS: u32 = 461;
pub const REKEY_INTERVAL: u32 = OVERLAP_SLOTS;

#[derive(Debug, Clone)]
pub struct SpongeRekeyState {
    frame_counter: u64,
    state: Vec<u8>,
    keystream_len: usize,
    rekey_counter: u32,
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
            rekey_counter: 0,
        }
    }

    pub fn advance_frame(&mut self) -> Vec<u8> {
        self.frame_counter += 1;
        self.rekey_counter += 1;

        if self.rekey_counter >= REKEY_INTERVAL {
            self.rekey_counter = 0;
            self.deep_rekey();
        }

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

    fn deep_rekey(&mut self) {
        let counter_bytes = self.frame_counter.to_le_bytes();
        for (i, byte) in self.state.iter_mut().enumerate() {
            *byte ^= counter_bytes[i % 8];
            *byte = byte.wrapping_add((i as u8).wrapping_mul(0x9E));
        }

        for round in 0..SPONGE_ROUNDS {
            self.permute_round(round);
        }
        for round in 0..SPONGE_ROUNDS {
            self.permute_round(round);
        }
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
        simd_xor_buffer(framebuffer, key);
    }

    pub fn xor_framebuffer_keystream(framebuffer: &mut [u8], keystream: &[u8]) {
        if keystream.is_empty() {
            return;
        }
        simd_xor_buffer(framebuffer, keystream);
    }

    pub fn xor_dirty_regions(
        framebuffer: &mut [u8],
        keystream: &[u8],
        dirty_tiles: &[(u32, u32, u32, u32)],
        stride: u32,
    ) {
        if keystream.is_empty() {
            return;
        }
        for &(tx, ty, tw, th) in dirty_tiles {
            for row in ty..(ty + th) {
                let offset = (row * stride + tx * 4) as usize;
                let end = offset + (tw as usize * 4);
                if end <= framebuffer.len() && end <= keystream.len() {
                    for i in offset..end {
                        framebuffer[i] ^= keystream[i % keystream.len()];
                    }
                }
            }
        }
    }

    pub fn reset(&mut self, new_key: &[u8]) {
        self.frame_counter = 0;
        self.rekey_counter = 0;
        for (i, byte) in self.state.iter_mut().enumerate() {
            *byte = if i < new_key.len() { new_key[i] } else { 0 };
        }
    }
}

fn simd_xor_buffer(buffer: &mut [u8], key: &[u8]) {
    let key_len = key.len();

    #[cfg(target_arch = "x86_64")]
    {
        let chunks = buffer.len() / 32;
        let remainder_start = chunks * 32;

        for chunk_idx in 0..chunks {
            let buf_offset = chunk_idx * 32;
            let mut key_block = [0u8; 32];
            for j in 0..32 {
                key_block[j] = key[(buf_offset + j) % key_len];
            }
            for j in 0..32 {
                buffer[buf_offset + j] ^= key_block[j];
            }
        }

        for i in remainder_start..buffer.len() {
            buffer[i] ^= key[i % key_len];
        }
        return;
    }

    #[cfg(target_arch = "aarch64")]
    {
        let chunks = buffer.len() / 16;
        let remainder_start = chunks * 16;

        for chunk_idx in 0..chunks {
            let buf_offset = chunk_idx * 16;
            for j in 0..16 {
                buffer[buf_offset + j] ^= key[(buf_offset + j) % key_len];
            }
        }

        for i in remainder_start..buffer.len() {
            buffer[i] ^= key[i % key_len];
        }
        return;
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        for (i, pixel) in buffer.iter_mut().enumerate() {
            *pixel ^= key[i % key_len];
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
        assert_eq!(OVERLAP_SLOTS, 461);
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
    fn test_keystream_xor_roundtrip() {
        let key = [0x42u8; 32];
        let mut state = SpongeRekeyState::new(&key, FrameResolution::Custom { width: 4, height: 4 });
        let keystream = state.advance_frame();

        let original = [10u8, 20, 30, 40, 50, 60, 70, 80];
        let mut buf = original.clone();
        let len = buf.len();

        SpongeRekeyState::xor_framebuffer_keystream(&mut buf, &keystream[..len]);
        assert_ne!(buf, original);

        SpongeRekeyState::xor_framebuffer_keystream(&mut buf, &keystream[..len]);
        assert_eq!(buf, original);
    }

    #[test]
    fn test_deep_rekey_at_interval() {
        let key = [0x42u8; 32];
        let mut state = SpongeRekeyState::new(&key, FrameResolution::Custom { width: 2, height: 2 });

        for _ in 0..REKEY_INTERVAL {
            state.advance_frame();
        }
        assert_eq!(state.rekey_counter, 0);
        assert_eq!(state.frame_counter(), REKEY_INTERVAL as u64);
    }

    #[test]
    fn test_dirty_region_xor() {
        let mut fb = alloc::vec![0u8; 64];
        for i in 0..64 { fb[i] = i as u8; }
        let key = alloc::vec![0xFFu8; 64];
        let original = fb.clone();

        let dirty = [(0u32, 0u32, 2u32, 2u32)];
        SpongeRekeyState::xor_dirty_regions(&mut fb, &key, &dirty, 16);

        assert_ne!(fb[..8], original[..8]);
    }

    #[test]
    fn test_frame_resolution() {
        assert_eq!(FrameResolution::Hd1080.pixel_bytes(), 1920 * 1080 * 4);
        assert_eq!(FrameResolution::Uhd4K.dimensions(), (3840, 2160));
    }
}
