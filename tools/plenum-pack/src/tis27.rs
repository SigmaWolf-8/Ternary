const STATE_SIZE: usize = 729;
const RATE: usize = 243;
const ROUNDS_TIS: usize = 4;
const LANES: usize = 27;

#[inline(always)]
fn balanced_wrap(s: i8) -> i8 {
    if s >= 2 {
        s - 3
    } else if s <= -2 {
        s + 3
    } else {
        s
    }
}

#[inline(always)]
fn trit_add(a: i8, b: i8) -> i8 {
    let s = a + b;
    if s > 1 {
        s - 3
    } else if s < -1 {
        s + 3
    } else {
        s
    }
}

#[inline(always)]
const fn gf3_mul(a: u8, b: u8) -> u8 {
    (a * b) % 3
}

#[inline(always)]
const fn gf3_add(a: u8, b: u8) -> u8 {
    (a + b) % 3
}

#[inline(always)]
const fn gf27_mul(a: [u8; 3], b: [u8; 3]) -> [u8; 3] {
    let c0 = gf3_mul(a[0], b[0]);
    let c1 = gf3_add(gf3_mul(a[0], b[1]), gf3_mul(a[1], b[0]));
    let c2 = gf3_add(
        gf3_add(gf3_mul(a[0], b[2]), gf3_mul(a[1], b[1])),
        gf3_mul(a[2], b[0]),
    );
    let c3 = gf3_add(gf3_mul(a[1], b[2]), gf3_mul(a[2], b[1]));
    let c4 = gf3_mul(a[2], b[2]);
    [
        gf3_add(c0, gf3_mul(2, c3)),
        gf3_add(gf3_add(c1, c3), gf3_mul(2, c4)),
        gf3_add(c2, c4),
    ]
}

#[inline(always)]
const fn gf27_pow17(x: [u8; 3]) -> [u8; 3] {
    let x2 = gf27_mul(x, x);
    let x4 = gf27_mul(x2, x2);
    let x8 = gf27_mul(x4, x4);
    let x16 = gf27_mul(x8, x8);
    gf27_mul(x16, x)
}

#[inline(always)]
const fn gf27_affine(x: [u8; 3]) -> [u8; 3] {
    let p = gf27_pow17(x);
    [
        gf3_add(
            gf3_add(p[0], gf3_add(p[1], gf3_mul(2, p[2]))),
            1,
        ),
        gf3_add(gf3_add(gf3_mul(2, p[0]), p[1]), p[2]),
        gf3_add(
            gf3_add(p[0], gf3_add(gf3_mul(2, p[1]), p[2])),
            2,
        ),
    ]
}

static CHI_MAP: [[i8; 3]; 27] = {
    let mut map = [[0i8; 3]; 27];
    let mut idx = 0usize;
    while idx < 27 {
        let [r0, r1, r2] = gf27_affine([
            (idx % 3) as u8,
            ((idx / 3) % 3) as u8,
            (idx / 9) as u8,
        ]);
        map[idx] = [r0 as i8 - 1, r1 as i8 - 1, r2 as i8 - 1];
        idx += 1;
    }
    map
};

static PERM: [u16; STATE_SIZE] = {
    let mut p = [0u16; STATE_SIZE];
    let mut i = 0usize;
    while i < STATE_SIZE {
        p[i] = ((i * 376 + 1) % STATE_SIZE) as u16;
        i += 1;
    }
    p
};

static RC_TABLE: [[i8; LANES]; ROUNDS_TIS] = {
    let mut rc = [[0i8; LANES]; ROUNDS_TIS];
    let mut r = 0usize;
    while r < ROUNDS_TIS {
        let mut lane = 0usize;
        while lane < LANES {
            rc[r][lane] = ((r * 7 + lane * 13 + 3) % 3) as i8 - 1;
            lane += 1;
        }
        r += 1;
    }
    rc
};

#[derive(Copy, Clone)]
struct ThetaNeighbors {
    left: [u16; 3],
    right: [u16; 3],
}

static THETA_IDX: [ThetaNeighbors; STATE_SIZE] = {
    let mut t = [ThetaNeighbors {
        left: [0; 3],
        right: [0; 3],
    }; STATE_SIZE];
    let w = STATE_SIZE;
    let mut i = 0;
    while i < w {
        t[i] = ThetaNeighbors {
            left: [
                ((i + w - 13) % w) as u16,
                ((i + w - 7) % w) as u16,
                ((i + w - 1) % w) as u16,
            ],
            right: [
                ((i + 1) % w) as u16,
                ((i + 7) % w) as u16,
                ((i + 13) % w) as u16,
            ],
        };
        i += 1;
    }
    t
};

fn chi_layer(state: &mut [i8; STATE_SIZE]) {
    let mut block = 0;
    while block < STATE_SIZE {
        let idx = (state[block] + 1) as usize
            + (state[block + 1] + 1) as usize * 3
            + (state[block + 2] + 1) as usize * 9;
        let r = CHI_MAP[idx];
        state[block] = r[0];
        state[block + 1] = r[1];
        state[block + 2] = r[2];
        block += 3;
    }
}

fn theta_pi_rc(state: &mut [i8; STATE_SIZE], buf: &mut [i8; STATE_SIZE], round: usize) {
    for i in 0..STATE_SIZE {
        let n = &THETA_IDX[i];
        let left = balanced_wrap(
            state[n.left[0] as usize]
                + state[n.left[1] as usize]
                + state[n.left[2] as usize],
        );
        let right = balanced_wrap(
            state[n.right[0] as usize]
                + state[n.right[1] as usize]
                + state[n.right[2] as usize],
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

fn permute(state: &mut [i8; STATE_SIZE]) {
    let mut buf = [0i8; STATE_SIZE];
    for round in 0..ROUNDS_TIS {
        chi_layer(state);
        theta_pi_rc(state, &mut buf, round);
    }
}

fn bytes_to_trits(bytes: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(bytes.len() * 5);
    for &byte in bytes {
        let mut v = byte;
        for _ in 0..5 {
            trits.push((v % 3) as i8 - 1);
            v /= 3;
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
                val += (trits[i + j] + 1) as u8 * pow;
            }
            pow = pow.wrapping_mul(3);
        }
        bytes.push(val);
        i += 5;
    }
    bytes
}

pub fn hash_hex_tis(input: &[u8]) -> String {
    let trits = bytes_to_trits(input);
    let mut state = [0i8; STATE_SIZE];

    let mut offset = 0;
    while offset + RATE <= trits.len() {
        for i in 0..RATE {
            state[i] = trit_add(state[i], trits[offset + i]);
        }
        permute(&mut state);
        offset += RATE;
    }

    let remaining = trits.len() - offset;
    for i in 0..remaining {
        state[i] = trit_add(state[i], trits[offset + i]);
    }
    if remaining < RATE {
        state[remaining] = trit_add(state[remaining], 1);
    }
    permute(&mut state);

    let output_trits: Vec<i8> = state[..243].to_vec();
    let output_bytes = trits_to_bytes(&output_trits);
    output_bytes[..49]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_deterministic() {
        assert_eq!(hash_hex_tis(b"hello world"), hash_hex_tis(b"hello world"));
    }

    #[test]
    fn hash_different_inputs() {
        assert_ne!(hash_hex_tis(b"hello"), hash_hex_tis(b"world"));
    }

    #[test]
    fn hash_empty_input() {
        let h = hash_hex_tis(b"");
        assert_eq!(h.len(), 98);
    }

    #[test]
    fn hash_hex_length() {
        let h = hash_hex_tis(b"test data for checksum");
        assert_eq!(h.len(), 98);
    }
}
