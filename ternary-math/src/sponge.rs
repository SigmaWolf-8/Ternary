// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TL-Sponge-385 — Standalone Post-Quantum Sponge Construction (v3 — Chi Layer)
// Location: ternary-math/src/sponge.rs
//
// Mirrors the kernel sponge (src/kernel/src/crypto/sponge.rs) as a
// standalone std-compatible implementation for use by inter-cube and
// other non-kernel crates that need 385-bit PQ security.
//
// Sponge version history:
//   v1: 3-neighbor theta, 27 rounds (deprecated)
//   v2: 7-neighbor theta (±1,±7,±13), 9 rounds, no chi
//   v3: Chi layer χ(x) = x¹⁷ over GF(27) added before theta
//
// Round function (v3):
//   1. Chi — χ(x) = x¹⁷ over GF(27) on each 3-trit block (243 blocks)
//   2. Theta — 7-neighbor extended substitution (±1, ±7, ±13)
//   3. Pi — stride-376 fixed permutation
//   4. Round constants
//
// Parameters:
//   State: 729 trits | Rate: 243 | Capacity: 486 (385-bit PQ security)
//   Rounds: 9 (3² — 3× safety margin over 3-round full diffusion)
//   Theta: 7-neighbor extended (±1, ±7, ±13) — all coprime to 729
//   Diffusion: π(i) = (376·i + 1) mod 729 — full-period permutation
//   Round constants: (7·round + 13·lane + 3) mod 3 − 1

const STATE_SIZE: usize = 729;
const RATE: usize = 243;
const ROUNDS: usize = 9;
const LANES: usize = 27;
const CHI_BLOCKS: usize = 243;

pub const SPONGE_VERSION: u8 = 2;

#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 { s - 3 } else if s <= -2 { s + 3 } else { s }
}

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 { s - 3 } else if s < -1 { s + 3 } else { s }
}

#[inline(always)]
fn gf3_mul(a: u8, b: u8) -> u8 {
    (a * b) % 3
}

#[inline(always)]
fn gf3_add(a: u8, b: u8) -> u8 {
    (a + b) % 3
}

#[inline(always)]
fn gf27_mul(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    let c0 = gf3_mul(a[0], b[0]);
    let c1 = gf3_add(gf3_mul(a[0], b[1]), gf3_mul(a[1], b[0]));
    let c2 = gf3_add(gf3_add(gf3_mul(a[0], b[2]), gf3_mul(a[1], b[1])), gf3_mul(a[2], b[0]));
    let c3 = gf3_add(gf3_mul(a[1], b[2]), gf3_mul(a[2], b[1]));
    let c4 = gf3_mul(a[2], b[2]);

    let r0 = gf3_add(c0, gf3_mul(2, c3));
    let r1 = gf3_add(gf3_add(c1, c3), gf3_mul(2, c4));
    let r2 = gf3_add(c2, c4);

    [r0, r1, r2]
}

#[inline(always)]
fn gf27_pow17(x: [u8; 3]) -> [u8; 3] {
    let x2 = gf27_mul(x, x);
    let x4 = gf27_mul(x2, x2);
    let x8 = gf27_mul(x4, x4);
    let x16 = gf27_mul(x8, x8);
    gf27_mul(x16, x)
}

fn chi_layer(state: &mut [i8; STATE_SIZE]) {
    for b in 0..CHI_BLOCKS {
        let base = b * 3;
        let g0 = (state[base] + 1) as u8;
        let g1 = (state[base + 1] + 1) as u8;
        let g2 = (state[base + 2] + 1) as u8;

        if g0 == 0 && g1 == 0 && g2 == 0 { continue; }

        let [r0, r1, r2] = gf27_pow17([g0, g1, g2]);
        state[base] = r0 as i8 - 1;
        state[base + 1] = r1 as i8 - 1;
        state[base + 2] = r2 as i8 - 1;
    }
}

static PERM: [u16; STATE_SIZE] = {
    let mut p = [0u16; STATE_SIZE];
    let mut i = 0usize;
    while i < STATE_SIZE {
        p[i] = ((i * 376 + 1) % STATE_SIZE) as u16;
        i += 1;
    }
    p
};

static RC_TABLE: [[i8; LANES]; ROUNDS] = {
    let mut rc = [[0i8; LANES]; ROUNDS];
    let mut r = 0usize;
    while r < ROUNDS {
        let mut lane = 0usize;
        while lane < LANES {
            let val = (r * 7 + lane * 13 + 3) % 3;
            rc[r][lane] = val as i8 - 1;
            lane += 1;
        }
        r += 1;
    }
    rc
};

fn theta_pi_rc(state: &mut [i8; STATE_SIZE], buf: &mut [i8; STATE_SIZE], round: usize) {
    let w = STATE_SIZE;
    for i in 0..w {
        let left = balanced_wrap(
            state[(i + w - 13) % w] +
            state[(i + w - 7) % w] +
            state[(i + w - 1) % w]
        );
        let right = balanced_wrap(
            state[(i + 1) % w] +
            state[(i + 7) % w] +
            state[(i + 13) % w]
        );
        buf[i] = balanced_wrap(left + state[i] + right + 1);
    }

    for i in 0..STATE_SIZE {
        state[PERM[i] as usize] = buf[i];
    }

    let rc = &RC_TABLE[round];
    for lane in 0..LANES {
        let idx = lane * LANES;
        state[idx] = balanced_wrap(state[idx] + rc[lane]);
    }
}

fn sponge_permutation(state: &mut [i8; STATE_SIZE]) {
    let mut buf = [0i8; STATE_SIZE];
    for round in 0..ROUNDS {
        chi_layer(state);
        theta_pi_rc(state, &mut buf, round);
    }
}

pub fn sponge_permutation_v1(state: &mut [i8; STATE_SIZE]) {
    let mut buf = [0i8; STATE_SIZE];
    for round in 0..ROUNDS {
        theta_pi_rc(state, &mut buf, round);
    }
}

pub fn bytes_to_trits_pub(bytes: &[u8]) -> Vec<i8> {
    bytes_to_trits(bytes)
}

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes {
        let mut val = byte;
        for _ in 0..5 {
            trits.push((val % 3) as i8 - 1);
            val /= 3;
        }
    }
    trits
}

fn trits_to_bytes(trits: &[i8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((trits.len() + 4) / 5);
    let mut i = 0;
    while i < trits.len() {
        let mut val: u8 = 0;
        let mut pow: u8 = 1;
        for j in 0..5 {
            if i + j < trits.len() {
                let t = (trits[i + j] + 1) as u8;
                val += t * pow;
            }
            pow = pow.wrapping_mul(3);
        }
        bytes.push(val);
        i += 5;
    }
    bytes
}

pub struct Sponge385Pub {
    state: [i8; STATE_SIZE],
    buf: [i8; RATE],
    buf_len: usize,
    absorbed: bool,
}

impl Sponge385Pub {
    pub fn new() -> Self {
        Self {
            state: [0i8; STATE_SIZE],
            buf: [0i8; RATE],
            buf_len: 0,
            absorbed: false,
        }
    }

    pub fn absorb(&mut self, input: &[i8]) {
        self.absorbed = true;
        let mut offset = 0;
        let input_len = input.len();

        if self.buf_len > 0 {
            let space = RATE - self.buf_len;
            let fill = if input_len < space { input_len } else { space };
            self.buf[self.buf_len..self.buf_len + fill]
                .copy_from_slice(&input[..fill]);
            self.buf_len += fill;
            offset = fill;

            if self.buf_len == RATE {
                for i in 0..RATE {
                    self.state[i] = trit_add(self.state[i], self.buf[i]);
                }
                sponge_permutation(&mut self.state);
                self.buf_len = 0;
            }
        }

        while offset + RATE <= input_len {
            let block = &input[offset..offset + RATE];
            for i in 0..RATE {
                self.state[i] = trit_add(self.state[i], block[i]);
            }
            sponge_permutation(&mut self.state);
            offset += RATE;
        }

        let remaining = input_len - offset;
        if remaining > 0 {
            self.buf[self.buf_len..self.buf_len + remaining]
                .copy_from_slice(&input[offset..]);
            self.buf_len += remaining;
        }
    }

    pub fn absorb_bytes(&mut self, input: &[u8]) {
        let trits = bytes_to_trits(input);
        self.absorb(&trits);
    }

    pub fn squeeze(&mut self, output_trits: usize) -> Vec<i8> {
        if self.buf_len > 0 || !self.absorbed {
            for i in 0..self.buf_len {
                self.state[i] = trit_add(self.state[i], self.buf[i]);
            }
            if self.buf_len < RATE {
                self.state[self.buf_len] = trit_add(self.state[self.buf_len], 1);
            }
            self.buf_len = 0;
            sponge_permutation(&mut self.state);
        }

        let mut output = Vec::with_capacity(output_trits);
        while output.len() < output_trits {
            let remaining = output_trits - output.len();
            let take = if remaining < RATE { remaining } else { RATE };
            output.extend_from_slice(&self.state[..take]);
            if output.len() < output_trits {
                sponge_permutation(&mut self.state);
            }
        }
        output.truncate(output_trits);
        output
    }
}

pub fn hash(input: &[u8], output_len: usize) -> Vec<u8> {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(input);
    let output_trits = output_len * 5;
    let trits = sponge.squeeze(output_trits);
    let bytes = trits_to_bytes(&trits);
    bytes[..output_len].to_vec()
}

pub fn derive_key(context: &[u8], material: &[u8], key_len: usize) -> Vec<u8> {
    let mut input = Vec::with_capacity(context.len() + material.len());
    input.extend_from_slice(context);
    input.extend_from_slice(material);
    hash(&input, key_len)
}

pub fn sponge385_derive_key(
    domain: &[u8],
    addr_a: &[u8],
    addr_b: &[u8],
    kem_shared_secret: &[u8; 32],
    epoch: u64,
) -> Vec<u8> {
    let mut sponge = Sponge385Pub::new();
    sponge.absorb_bytes(domain);
    sponge.absorb_bytes(addr_a);
    sponge.absorb_bytes(addr_b);
    sponge.absorb_bytes(kem_shared_secret);
    sponge.absorb_bytes(&epoch.to_le_bytes());
    let trits = sponge.squeeze(RATE);
    let bytes = trits_to_bytes(&trits);
    bytes[..32].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let a = hash(b"hello world", 32);
        let b = hash(b"hello world", 32);
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_different_inputs() {
        let a = hash(b"hello", 32);
        let b = hash(b"world", 32);
        assert_ne!(a, b);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let a = derive_key(b"ctx", b"material", 32);
        let b = derive_key(b"ctx", b"material", 32);
        assert_eq!(a, b);
    }

    #[test]
    fn test_derive_key_length() {
        let k = derive_key(b"ctx", b"material", 32);
        assert_eq!(k.len(), 32);
    }

    #[test]
    fn test_sponge385_derive_key_symmetric() {
        let addr_a = b"addr_a_bytes";
        let addr_b = b"addr_b_bytes";
        let secret = [42u8; 32];
        let epoch = 100u64;
        let k1 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, epoch,
        );
        let k2 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, epoch,
        );
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 32);
    }

    #[test]
    fn test_sponge385_derive_key_different_secrets() {
        let addr_a = b"addr_a_bytes";
        let addr_b = b"addr_b_bytes";
        let secret1 = [42u8; 32];
        let secret2 = [99u8; 32];
        let k1 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret1, 100,
        );
        let k2 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret2, 100,
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_sponge385_derive_key_different_epochs() {
        let addr_a = b"addr_a_bytes";
        let addr_b = b"addr_b_bytes";
        let secret = [42u8; 32];
        let k1 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, 100,
        );
        let k2 = sponge385_derive_key(
            b"PlenumNET-CON-v3.0", addr_a, addr_b, &secret, 200,
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_perm_full_period() {
        let mut seen = [false; STATE_SIZE];
        for i in 0..STATE_SIZE {
            let dest = PERM[i] as usize;
            assert!(!seen[dest]);
            seen[dest] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn test_neighbor_distances_coprime() {
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 { let t = b; b = a % b; a = t; } a
        }
        assert_eq!(gcd(1, STATE_SIZE), 1);
        assert_eq!(gcd(7, STATE_SIZE), 1);
        assert_eq!(gcd(13, STATE_SIZE), 1);
    }

    #[test]
    fn test_chi_is_bijection() {
        let mut outputs = Vec::new();
        for a0 in 0u8..3 {
            for a1 in 0u8..3 {
                for a2 in 0u8..3 {
                    let result = gf27_pow17([a0, a1, a2]);
                    outputs.push(result);
                }
            }
        }
        assert_eq!(gf27_pow17([0, 0, 0]), [0, 0, 0], "0 must map to 0");
        outputs.sort();
        outputs.dedup();
        assert_eq!(outputs.len(), 27, "chi must be a bijection over all 27 GF(27) elements");
    }

    #[test]
    fn test_chi_v2_differs_from_v1() {
        let mut s1 = [0i8; STATE_SIZE];
        let mut s2 = [0i8; STATE_SIZE];
        s1[0] = 1; s1[1] = -1; s1[3] = 1;
        s2 = s1;
        sponge_permutation_v1(&mut s1);
        sponge_permutation(&mut s2);
        assert_ne!(s1, s2, "v2 (with chi) must produce different output from v1");
    }
}
