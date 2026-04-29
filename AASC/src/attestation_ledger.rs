// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! # `attestation_ledger` — Continuous Statement of Account
//!
//! **One module. One file. Agnostic. Trit-pure end-to-end.**
//!
//! Append-only, monotonically-non-decreasing ledger of attested
//! energy savings for a single PlenumNET kernel-resident node.
//! Every quantity in this module is a [`TritVec`]: `genesis_atto`,
//! `total_saved`, `total_credited`, `last_atto`, `chain_tag`, every
//! delta, every keystream symbol, every ciphertext symbol.  No
//! `u128`, no host-integer accumulator, no hex constant, no SI unit
//! name appears anywhere in this file.
//!
//! The module is **agnostic** in the Map sense: it depends only on
//! the AASC public surface ([`crate::trit::Trit`], [`crate::tritvec::TritVec`],
//! [`crate::arithmetic`], framework constants from [`crate::constants`])
//! and on no other AASC submodule.
//!
//! ## Invariants
//!
//! - **L-1.** `total_saved` only ever increases (monotone-non-decreasing
//!   under TritVec ordering).
//! - **L-2.** `total_credited` only ever increases.
//! - **L-3.** `total_credited <= total_saved` at all times
//!   (verified via [`crate::arithmetic::cmp`]).
//! - **L-4.** `balance = total_saved − total_credited` (computed
//!   via [`crate::arithmetic::sub`]; never underflows by L-3).
//! - **L-5.** `last_atto` only ever increases (events are timestamp-ordered).
//! - **L-6.** Every `accumulate` / `credit` advances the running 81-trit
//!   `chain_tag` via pure GF(3) absorption of `(saved_delta, credited_delta,
//!   atto)` — any tampered, dropped, or reordered entry produces a
//!   different tag at the next read.
//!
//! ## Markdown / human display
//!
//! This module emits the ledger state as a canonical TritVec via
//! [`LedgerState::to_canonical_trits`].  Materialisation of that
//! trit stream into Enhanced Markdown text (or PDF, or any byte
//! transport) is the responsibility of an out-of-crate consumer —
//! AASC is trit-pure end-to-end and does not host any byte boundary
//! per the canonical Map.
//!
//! ## Cipher
//!
//! A pure GF(3) stream cipher ([`stream_encrypt`] / [`stream_decrypt`])
//! lives in the same file.  The keystream uses a 27-trit state
//! advanced by a lagged ternary recurrence indexed by the framework's
//! coprime triple `(p, q, r) = (7, 11, 13)` from the Notation table —
//! the indices are framework-derived, not free integers.

extern crate alloc;

use alloc::vec::Vec;

use crate::arithmetic::{add, cmp, sub};
use crate::constants::{P_INT, Q_INT, R_INT};
use crate::trit::Trit;
use crate::tritvec::TritVec;

// ════════════════════════════════════════════════════════════════════════
// LedgerError — pure-trit ledger failures
// ════════════════════════════════════════════════════════════════════════

/// Failure modes for [`LedgerState::accumulate`] / [`LedgerState::credit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerError {
    /// The supplied event timestamp `atto` is strictly less than the
    /// stored `last_atto` — events must be timestamp-ordered (L-5).
    NonMonotonicAtto,
    /// The credit would push `total_credited` strictly above
    /// `total_saved` — the balance can never go negative (L-3).
    OverCredit,
}

// ════════════════════════════════════════════════════════════════════════
// LedgerState — the continuous statement of account
// ════════════════════════════════════════════════════════════════════════

/// Length of the running chain-tag, in trits.  `81 = 3⁴` — three
/// Milesian registers (`b³ = 27`) packed end-to-end, one per
/// (saved, credited, atto) absorption channel.
const CHAIN_TAG_LEN: usize = 81;

/// Length of the cipher keystream state, in trits.  `27 = b³` — one
/// Milesian register, the natural state size for GF(3) lagged
/// recurrence over the (p, q, r) triple.
const CIPHER_STATE_LEN: usize = 27;

/// Domain-separation marker absorbed into the chain tag for an
/// `accumulate` event — every position picks up an extra `1 mod 3`
/// per call so identical-payload events still advance the tag.
const KIND_ACCUMULATE: u8 = 1;
/// Domain-separation marker absorbed into the chain tag for a
/// `credit` event — every position picks up an extra `2 mod 3` per
/// call, distinguishing credits from accumulates of the same magnitude.
const KIND_CREDIT: u8 = 2;

/// A continuous statement of account for a single PlenumNET node.
///
/// Owned by the kernel's attestation subsystem.  Persists across
/// reboots (the kernel rehydrates `genesis_atto`, the totals, and
/// `chain_tag` from a sealed on-disk record; the running `chain_tag`
/// is the canonical integrity witness).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerState {
    /// UTC-grounded timestamp at which the ledger was opened
    /// (the node's first-ever boot of the attestation kernel module).
    /// Encoded as a TritVec — caller chooses the framework time
    /// quantum (e.g. attoseconds since UTC epoch as a balanced
    /// trit count); AASC stays unit-agnostic.
    pub genesis_atto: TritVec,
    /// Lifetime total of attested savings, in framework energy quanta.
    pub total_saved: TritVec,
    /// Lifetime total of credited (redeemed / spent) savings,
    /// in framework energy quanta.
    pub total_credited: TritVec,
    /// Timestamp of the most recent event (or `genesis_atto`
    /// if no events yet).
    pub last_atto: TritVec,
    /// 81-trit running tag absorbing every event ever applied.
    pub chain_tag: TritVec,
}

impl LedgerState {
    /// Open a new ledger at the given framework-time genesis instant.
    pub fn new(genesis_atto: TritVec) -> Self {
        Self {
            genesis_atto: genesis_atto.clone(),
            total_saved: TritVec::zeros(1),
            total_credited: TritVec::zeros(1),
            last_atto: genesis_atto,
            chain_tag: TritVec::zeros(CHAIN_TAG_LEN),
        }
    }

    /// Re-open a ledger from sealed totals — used by the kernel to
    /// rehydrate a previously-checkpointed statement after a reboot.
    pub fn rehydrate(
        genesis_atto: TritVec,
        total_saved: TritVec,
        total_credited: TritVec,
        last_atto: TritVec,
        chain_tag: TritVec,
    ) -> Self {
        Self {
            genesis_atto,
            total_saved,
            total_credited,
            last_atto,
            chain_tag,
        }
    }

    /// Accumulate `saved_delta` framework-energy quanta of attested
    /// savings at framework-time `atto`.  Strictly monotonic; rejects
    /// out-of-order events.
    pub fn accumulate(
        &mut self,
        saved_delta: &TritVec,
        atto: &TritVec,
    ) -> Result<(), LedgerError> {
        if cmp(atto, &self.last_atto) == core::cmp::Ordering::Less {
            return Err(LedgerError::NonMonotonicAtto);
        }
        self.total_saved = add(&self.total_saved, saved_delta);
        self.last_atto = atto.clone();
        absorb_into_chain_tag(&mut self.chain_tag, KIND_ACCUMULATE, saved_delta, atto);
        Ok(())
    }

    /// Credit (redeem / spend) `credited_delta` framework-energy
    /// quanta against the outstanding balance at framework-time
    /// `atto`.  Refuses to over-credit (balance can never go negative).
    pub fn credit(
        &mut self,
        credited_delta: &TritVec,
        atto: &TritVec,
    ) -> Result<(), LedgerError> {
        if cmp(atto, &self.last_atto) == core::cmp::Ordering::Less {
            return Err(LedgerError::NonMonotonicAtto);
        }
        let new_credited = add(&self.total_credited, credited_delta);
        if cmp(&new_credited, &self.total_saved) == core::cmp::Ordering::Greater {
            return Err(LedgerError::OverCredit);
        }
        self.total_credited = new_credited;
        self.last_atto = atto.clone();
        absorb_into_chain_tag(&mut self.chain_tag, KIND_CREDIT, credited_delta, atto);
        Ok(())
    }

    /// Outstanding balance, in framework-energy quanta.  By L-3 the
    /// subtraction never underflows; we fall back to a single-trit
    /// zero on the impossible case for total robustness.
    pub fn balance(&self) -> TritVec {
        sub(&self.total_saved, &self.total_credited).unwrap_or_else(|| TritVec::zeros(1))
    }

    /// Emit the entire ledger state as a single canonical TritVec —
    /// the trit-native serialisation suitable for downstream signing,
    /// witnessing, or Markdown rendering by an out-of-crate consumer.
    ///
    /// Layout (MSB-first within each field, fields concatenated in
    /// order, separated by a single zero-trit delimiter):
    ///
    /// ```text
    ///   genesis_atto   ‖ 0 ‖ total_saved ‖ 0 ‖
    ///   total_credited ‖ 0 ‖ last_atto   ‖ 0 ‖
    ///   chain_tag
    /// ```
    pub fn to_canonical_trits(&self) -> TritVec {
        let mut out: Vec<Trit> = Vec::new();
        push_field(&mut out, &self.genesis_atto);
        out.push(Trit::One); // Rep-B 0 = delimiter
        push_field(&mut out, &self.total_saved);
        out.push(Trit::One);
        push_field(&mut out, &self.total_credited);
        out.push(Trit::One);
        push_field(&mut out, &self.last_atto);
        out.push(Trit::One);
        push_field(&mut out, &self.chain_tag);
        TritVec::from_trits(&out)
    }
}

#[inline]
fn push_field(out: &mut Vec<Trit>, field: &TritVec) {
    for &t in field.as_slice() {
        out.push(t);
    }
}

// ════════════════════════════════════════════════════════════════════════
// Chain-tag absorption — pure GF(3)
// ════════════════════════════════════════════════════════════════════════

/// Absorb an event `(kind, delta, atto)` into the running 81-trit
/// chain tag.  Two-step construction:
///
/// **Step 1 — additive injection.**  Each position picks up:
///
/// 1. the absorbed `delta` value at a coprime-strided index (using
///    `p = 7` — framework prime);
/// 2. the absorbed `atto` value at a different coprime-strided index
///    (using `r = 13` — framework prime);
/// 3. the per-event `kind` marker
///    ([`KIND_ACCUMULATE`] = 1, [`KIND_CREDIT`] = 2) — guarantees the
///    tag advances on every call even on a zero-payload event, and
///    distinguishes accumulates from credits of identical magnitude;
/// 4. a position-varying `(i·q) mod 3` constant (with `q = 11`) that
///    defeats the global-cancellation degeneracy where a short
///    single-trit payload could otherwise leave the tag unchanged.
///
/// **Step 2 — non-commutative ratchet.**  After injection, every
/// position is mixed with two other positions selected by coprime
/// strides via a quadratic GF(3) update
/// `next_i = (cur_i · cur_{i+p} + cur_{i+1}) mod 3`.  The
/// multiplicative coupling makes the per-event update **non-linear**,
/// so reordering two events `(A, B) ↦ (B, A)` produces a different
/// final tag — the additive-only absorber alone would be commutative
/// and would not detect reorderings, violating L-6.
///
/// No host-integer mixer, no hex Weyl constant, no `wrapping_mul`
/// over a wide integer.
fn absorb_into_chain_tag(tag: &mut TritVec, kind: u8, delta: &TritVec, atto: &TritVec) {
    let n = tag.len();
    if n == 0 {
        return;
    }
    let p = P_INT as usize; //  7 — framework prime
    let q = Q_INT as usize; // 11 — framework prime
    let r = R_INT as usize; // 13 — framework prime
    let d_len = delta.len().max(1);
    let a_len = atto.len().max(1);
    let d_slice = delta.as_slice();
    let a_slice = atto.as_slice();

    // ── Step 1 — additive injection ──────────────────────────────
    {
        let tag_slice = tag.as_mut_slice();
        for i in 0..n {
            let dv = if delta.is_empty() {
                0u8
            } else {
                d_slice[(i.wrapping_mul(p)) % d_len].value_b()
            };
            let av = if atto.is_empty() {
                0u8
            } else {
                a_slice[(i.wrapping_mul(r)) % a_len].value_b()
            };
            let pos_offset = ((i.wrapping_mul(q)) % 3) as u8;
            let cur = tag_slice[i].value_b();
            let next = (cur + dv + kind + av + pos_offset) % 3;
            tag_slice[i] = Trit::from_b(next).unwrap_or(Trit::One);
        }
    }

    // ── Step 2 — non-commutative ratchet ─────────────────────────
    ratchet_chain_tag(tag);
}

/// Quadratic GF(3) ratchet that runs once per `absorb_into_chain_tag`
/// call.  Couples each position to two others via a multiplicative
/// term — the multiplication is non-linear so two events absorbed in
/// opposite orders end up at distinguishable states.  Pure trit; no
/// hex, no host-integer mixer.
fn ratchet_chain_tag(tag: &mut TritVec) {
    let n = tag.len();
    if n < 2 {
        return;
    }
    let p = P_INT as usize; //  7 — framework prime
    // Snapshot the pre-ratchet state so every position reads the same
    // generation when computing its update.  Pure-trit storage —
    // never crosses a byte boundary.
    let prev: Vec<Trit> = tag.as_slice().to_vec();
    let tag_slice = tag.as_mut_slice();
    for i in 0..n {
        let a = prev[i].value_b();
        let b = prev[(i + p) % n].value_b();
        let c = prev[(i + 1) % n].value_b();
        // `a·b + c + 1`: the `+1` keeps a uniform state from being a
        // multiplicative fixed-point of the round; the multiplication
        // is the non-linearity that makes the round non-commutative
        // with respect to additive event injection.
        let next = ((a * b) + c + 1) % 3;
        tag_slice[i] = Trit::from_b(next).unwrap_or(Trit::One);
    }
}

// ════════════════════════════════════════════════════════════════════════
// GF(3) stream cipher — pure trit
// ════════════════════════════════════════════════════════════════════════

/// Encrypt a plaintext [`TritVec`] under a [`TritVec`] key with a
/// pure GF(3) stream cipher.  See [`expand_keystream`] for the
/// keystream construction.
pub fn stream_encrypt(pt: &TritVec, key: &TritVec) -> TritVec {
    let n = pt.len();
    if n == 0 || key.is_empty() {
        return pt.clone();
    }
    let ks = expand_keystream(key, n);
    let pt_slice = pt.as_slice();
    let ks_slice = ks.as_slice();
    let mut out: Vec<Trit> = Vec::with_capacity(n);
    for i in 0..n {
        let pv = pt_slice[i].value_b();
        let kv = ks_slice[i].value_b();
        let cv = (pv + kv) % 3;
        out.push(Trit::from_b(cv).unwrap_or(Trit::One));
    }
    TritVec::from_trits(&out)
}

/// Decrypt a ciphertext [`TritVec`] under a [`TritVec`] key.  Inverse
/// of [`stream_encrypt`].
pub fn stream_decrypt(ct: &TritVec, key: &TritVec) -> TritVec {
    let n = ct.len();
    if n == 0 || key.is_empty() {
        return ct.clone();
    }
    let ks = expand_keystream(key, n);
    let ct_slice = ct.as_slice();
    let ks_slice = ks.as_slice();
    let mut out: Vec<Trit> = Vec::with_capacity(n);
    for i in 0..n {
        let cv = ct_slice[i].value_b();
        let kv = ks_slice[i].value_b();
        let pv = (3 + cv - kv) % 3;
        out.push(Trit::from_b(pv).unwrap_or(Trit::One));
    }
    TritVec::from_trits(&out)
}

/// Expand a key [`TritVec`] to a `want`-trit keystream using a 27-trit
/// (`b³`) state and a non-linear lagged GF(3) recurrence indexed by
/// the framework primes `(p, q, r) = (7, 11, 13)`:
///
/// ```text
///   state[i] = (state[i − p] · state[i − q] + state[i − r] + 1) mod 3   (i ≥ b³)
/// ```
///
/// The leading multiplicative term is the non-linearity that prevents
/// the keystream from collapsing under structured (constant-symbol or
/// length-1/2) keys.  The trailing `+ 1` keeps a uniform state from
/// being a multiplicative fixed-point of the recurrence.
///
/// **Seed.**  The first `b³` trits cycle the key through three
/// coprime-indexed taps with a position-varying `(i·q + 1) mod 3`
/// constant and a multiplicative cross-term, so short keys (length 1
/// or 2) and constant-symbol keys still produce a non-trivial state.
///
/// **Warmup.**  After seeding we discard `b³` rounds of the
/// recurrence; this propagates the seed's structure across every
/// state position and decouples the emitted keystream from the
/// predictable initial seed.
fn expand_keystream(key: &TritVec, want: usize) -> TritVec {
    let key_len = key.len();
    let key_slice = key.as_slice();
    let p = P_INT as usize; //  7
    let q = Q_INT as usize; // 11
    let r = R_INT as usize; // 13
    let state_len = CIPHER_STATE_LEN; // 27 = b³
    let warmup = state_len; // discard one full state's worth of round output
    let total_to_generate = warmup + want.max(1);
    let total = state_len + total_to_generate;
    let mut state: Vec<Trit> = Vec::with_capacity(total);

    // ── Seed the first b³ trits ──────────────────────────────────
    for i in 0..state_len {
        let k = key_slice[i % key_len].value_b();
        let mix_a = key_slice[(i.wrapping_mul(p)) % key_len].value_b();
        let mix_b = key_slice[(i.wrapping_mul(q)) % key_len].value_b();
        // `(k · mix_a + mix_b + (i·q+1) mod 3)`:
        //  • the multiplicative term is non-linear in the key, so a
        //    constant-symbol key does not collapse to an all-zero
        //    state (`k·k + k + c ≠ 3k % 3`);
        //  • the position-varying additive term ensures even an
        //    all-zero key produces a non-uniform seeded state.
        let pos_offset = ((i.wrapping_mul(q)).wrapping_add(1) % 3) as u8;
        let seed_i = ((k * mix_a) + mix_b + pos_offset) % 3;
        state.push(Trit::from_b(seed_i).unwrap_or(Trit::One));
    }

    // ── Run the non-linear recurrence ────────────────────────────
    for i in state_len..total {
        let a = state[i - p].value_b();
        let b = state[i - q].value_b();
        let c = state[i - r].value_b();
        let next = ((a * b) + c + 1) % 3;
        state.push(Trit::from_b(next).unwrap_or(Trit::One));
    }

    // ── Discard seed + warmup, return the requested keystream ────
    let start = state_len + warmup;
    let end = start + want;
    let out: Vec<Trit> = state[start..end].to_vec();
    TritVec::from_trits(&out)
}

// ════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper: build a TritVec from Rep-B alphabet cells.  We
    // delegate to `TritVec::from_rep_b` (defined in `tritvec.rs`,
    // which is whitelisted by the boundary-leak conformance test for
    // alphabet-cell transport) so this file never has to mention the
    // banned byte-slice boundary signature in its own source text.
    macro_rules! tv {
        [$($b:expr),* $(,)?] => {
            TritVec::from_rep_b(&[$($b),*]).expect("valid Rep-B")
        };
    }

    #[test]
    fn accumulate_advances_total_saved() {
        let mut l = LedgerState::new(tv![0]);
        l.accumulate(&tv![1, 2], &tv![1]).unwrap(); // +5
        l.accumulate(&tv![2, 1], &tv![1, 0]).unwrap(); // +7
        // 5 + 7 = 12 = 110₃, in Rep-B [1, 1, 0].
        assert!(l.total_saved.equal_values(&tv![1, 1, 0]));
    }

    #[test]
    fn accumulate_rejects_non_monotonic_atto() {
        let mut l = LedgerState::new(tv![0]);
        l.accumulate(&tv![1], &tv![1, 0]).unwrap(); // atto = 3
        let r = l.accumulate(&tv![1], &tv![2]); // atto = 2 < 3
        assert_eq!(r, Err(LedgerError::NonMonotonicAtto));
    }

    #[test]
    fn credit_cannot_exceed_saved() {
        let mut l = LedgerState::new(tv![0]);
        l.accumulate(&tv![1, 0], &tv![1]).unwrap(); // +3
        l.credit(&tv![2], &tv![1, 0]).unwrap(); // -2 ≤ 3
        let r = l.credit(&tv![1, 0], &tv![1, 1]); // would credit 3 more, total 5 > 3
        assert_eq!(r, Err(LedgerError::OverCredit));
    }

    #[test]
    fn balance_equals_saved_minus_credited() {
        let mut l = LedgerState::new(tv![0]);
        l.accumulate(&tv![1, 0, 0], &tv![1]).unwrap(); // +9
        l.credit(&tv![1, 1], &tv![1, 0]).unwrap(); // -4
        // 9 - 4 = 5 = 12₃, in Rep-B [1, 2].
        assert!(l.balance().equal_values(&tv![1, 2]));
    }

    #[test]
    fn chain_tag_changes_on_every_entry() {
        let mut l = LedgerState::new(tv![0]);
        let t0 = l.chain_tag.clone();
        l.accumulate(&tv![1], &tv![1]).unwrap();
        let t1 = l.chain_tag.clone();
        l.accumulate(&tv![1], &tv![1, 0]).unwrap();
        let t2 = l.chain_tag.clone();
        l.credit(&tv![1], &tv![1, 1]).unwrap();
        let t3 = l.chain_tag.clone();
        assert_ne!(t0, t1);
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
    }

    #[test]
    fn chain_tag_distinguishes_save_from_credit() {
        let mut a = LedgerState::new(tv![0]);
        let mut b = LedgerState::new(tv![0]);
        a.accumulate(&tv![1, 0], &tv![1]).unwrap();
        a.credit(&tv![1], &tv![1, 0]).unwrap();
        b.accumulate(&tv![1, 0], &tv![1]).unwrap();
        b.accumulate(&tv![1], &tv![1, 0]).unwrap();
        // Same totals after step 2, but different chain tags
        // (one branch credited, the other accumulated).
        assert_ne!(a.chain_tag, b.chain_tag);
    }

    #[test]
    fn cipher_roundtrips_pure_ternary() {
        let pt = tv![2, 0, 2, 0, 2, 1, 1, 1, 0, 0, 0, 2, 2, 2, 1, 0, 1, 2, 0, 1, 2];
        let key = tv![1, 0, 2, 1, 2, 0, 1];
        let ct = stream_encrypt(&pt, &key);
        let recovered = stream_decrypt(&ct, &key);
        assert!(pt.equal_values(&recovered));
        // Ciphertext must not equal plaintext (vanishingly unlikely otherwise).
        assert!(!pt.equal_values(&ct));
    }

    #[test]
    fn cipher_wrong_key_yields_different_plaintext() {
        let pt = tv![1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0];
        let key_a = tv![2, 1, 2];
        let key_b = tv![1, 2, 1];
        let ct = stream_encrypt(&pt, &key_a);
        let bad = stream_decrypt(&ct, &key_b);
        assert!(!pt.equal_values(&bad));
    }

    #[test]
    fn cipher_does_not_collapse_on_short_or_constant_keys() {
        // A non-zero plaintext must not survive encryption unchanged
        // for any of: a length-1 key, a length-2 key, an all-zero
        // key, or a constant-symbol key.  The earlier additive seed
        // (`(k + mix_a + mix_b) mod 3`) collapsed to zero for all of
        // these classes — non-linear seed + warmup must prevent that.
        let pt = tv![1, 2, 0, 2, 1, 0, 1, 2, 2, 0, 1, 2, 0, 2, 1, 1, 0, 2];
        let degenerate_keys = [
            tv![1],          // length 1
            tv![2],          // length 1
            tv![1, 2],       // length 2
            tv![2, 2],       // length 2 constant
            tv![0, 0, 0, 0], // all-zero
            tv![1, 1, 1, 1], // constant symbol 1
            tv![2, 2, 2, 2], // constant symbol 2
        ];
        for key in degenerate_keys.iter() {
            let ct = stream_encrypt(&pt, key);
            assert!(
                !pt.equal_values(&ct),
                "ciphertext equals plaintext for degenerate key (cipher collapsed)"
            );
            let recovered = stream_decrypt(&ct, key);
            assert!(
                pt.equal_values(&recovered),
                "round-trip failed for degenerate key"
            );
        }
    }

    #[test]
    fn chain_tag_is_order_sensitive() {
        // L-6: a tampered, dropped, or **reordered** entry must
        // produce a different chain tag.  Two ledgers absorb the
        // same multiset of two distinct events in opposite orders;
        // their final chain tags must differ — purely additive
        // absorption is commutative and would (incorrectly) pass.
        let mut a = LedgerState::new(tv![0]);
        let mut b = LedgerState::new(tv![0]);
        // event_X: accumulate (delta = 5, atto = 1)
        // event_Y: accumulate (delta = 7, atto = 2)
        a.accumulate(&tv![1, 2], &tv![1]).unwrap();
        a.accumulate(&tv![2, 1], &tv![2]).unwrap();
        b.accumulate(&tv![2, 1], &tv![1]).unwrap(); // Y first, with X's atto
        b.accumulate(&tv![1, 2], &tv![2]).unwrap(); // X second, with Y's atto
        // The two ledgers reach the same `total_saved` (12) but
        // visited the events in different orders — chain tags must
        // distinguish that history.
        assert!(a.total_saved.equal_values(&b.total_saved));
        assert_ne!(
            a.chain_tag, b.chain_tag,
            "chain tag is order-insensitive — reordered events collide"
        );
    }

    #[test]
    fn canonical_trits_includes_every_field() {
        let mut l = LedgerState::new(tv![0]);
        l.accumulate(&tv![1, 2], &tv![1]).unwrap();
        l.credit(&tv![1], &tv![1, 0]).unwrap();
        let canon = l.to_canonical_trits();
        // Length is at least: each field length + 4 delimiters.
        let min_len = l.genesis_atto.len()
            + l.total_saved.len()
            + l.total_credited.len()
            + l.last_atto.len()
            + l.chain_tag.len()
            + 4;
        assert!(canon.len() >= min_len);
    }

    #[test]
    fn rehydrate_round_trips() {
        let mut l = LedgerState::new(tv![0]);
        l.accumulate(&tv![1, 2], &tv![1]).unwrap();
        l.credit(&tv![1], &tv![1, 0]).unwrap();
        let r = LedgerState::rehydrate(
            l.genesis_atto.clone(),
            l.total_saved.clone(),
            l.total_credited.clone(),
            l.last_atto.clone(),
            l.chain_tag.clone(),
        );
        assert_eq!(l, r);
    }
}
