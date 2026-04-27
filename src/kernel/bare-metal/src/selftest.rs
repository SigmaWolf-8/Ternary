// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! Bare-Metal Self-Test Suite
//!
//! Exercises the ACTUAL plenumnet-kernel library on bare metal.
//! Every test here proves that real kernel code runs without an OS.

use crate::serial;

use plenumnet_kernel::ternary::{Trit, KernelTritExt, pack_trits, unpack_trits, Representation, convert_representation};
use plenumnet_kernel::arch::boot::{BootSequence, BootStage, x86_64_boot_config};
use plenumnet_kernel::arch::{ArchId, MemoryRegionType};
use plenumnet_kernel::timing::{FemtosecondTimestamp, FS_PER_NS, FS_PER_MS, FS_PER_SECOND};
use plenumnet_kernel::{SALVI_EPOCH_NS, SALVI_EPOCH_FS, KERNEL_VERSION};
use plenumnet_kernel::phase::EncryptionMode;
use plenumnet_kernel::Architecture;

pub struct TestResults {
    pub passed: u32,
    pub failed: u32,
}

impl TestResults {
    fn new() -> Self {
        TestResults { passed: 0, failed: 0 }
    }

    fn record(&mut self, name: &str, result: bool) {
        if result {
            serial::print_str("  [PASS] ");
            serial::print_line(name);
            self.passed += 1;
        } else {
            serial::print_str("  [FAIL] ");
            serial::print_line(name);
            self.failed += 1;
        }
    }
}

pub fn run_all() -> TestResults {
    let mut r = TestResults::new();

    // ── Kernel Init ────────────────────────────────────────────────
    serial::print_line("  -- Kernel Init --");

    let info = plenumnet_kernel::init();
    r.record("kernel version matches", info.version == KERNEL_VERSION);
    r.record("ternary_ops enabled", info.features.ternary_ops);
    r.record("femtosecond_timing enabled", info.features.femtosecond_timing);
    r.record("phase_encryption enabled", info.features.phase_encryption);

    // ── Salvi Epoch Constants ──────────────────────────────────────
    serial::print_line("  -- Salvi Epoch --");

    r.record("SALVI_EPOCH_NS is April 1 2025",
        SALVI_EPOCH_NS == 1_743_465_600_000_000_000);
    r.record("SALVI_EPOCH_FS consistent",
        SALVI_EPOCH_FS == SALVI_EPOCH_NS * 1_000_000);

    // ── Trit Arithmetic (GF(3)) ───────────────────────────────────
    serial::print_line("  -- Trit Arithmetic (GF(3)) --");

    let p = Trit::from_a(1).unwrap();
    let z = Trit::from_a(0).unwrap();
    let n = Trit::from_a(-1).unwrap();

    r.record("Trit::from_a(1) == +1", p.to_a() == 1);
    r.record("Trit::from_a(0) == 0", z.to_a() == 0);
    r.record("Trit::from_a(-1) == -1", n.to_a() == -1);
    r.record("Trit::from_a(5) == None", Trit::from_a(5).is_none());

    r.record("to_b: -1 -> 0", n.to_b() == 0);
    r.record("to_b:  0 -> 1", z.to_b() == 1);
    r.record("to_b: +1 -> 2", p.to_b() == 2);

    r.record("to_c: -1 -> 1", n.to_c() == 1);
    r.record("to_c:  0 -> 2", z.to_c() == 2);
    r.record("to_c: +1 -> 3", p.to_c() == 3);

    r.record("round-trip B: from_b(to_b(P))",
        Trit::from_b(p.to_b()).unwrap().to_a() == 1);
    r.record("round-trip C: from_c(to_c(N))",
        Trit::from_c(n.to_c()).unwrap().to_a() == -1);

    r.record("NOT(+1) == -1", p.not().to_a() == -1);
    r.record("NOT(-1) == +1", n.not().to_a() == 1);
    r.record("NOT(0) == 0", z.not().to_a() == 0);
    r.record("double negation: NOT(NOT(P)) == P",
        p.not().not().to_a() == p.to_a());

    r.record("AND(P, N) == N", p.and(&n).to_a() == -1);
    r.record("AND(P, P) == P", p.and(&p).to_a() == 1);
    r.record("AND(P, Z) == Z", p.and(&z).to_a() == 0);

    r.record("OR(P, N) == P", p.or(&n).to_a() == 1);
    r.record("OR(N, N) == N", n.or(&n).to_a() == -1);
    r.record("OR(N, Z) == Z", n.or(&z).to_a() == 0);

    r.record("add(P, P) in GF(3)", {
        let result = p.add(p);
        result.to_a() == -1
    });
    r.record("add(P, N) in GF(3)", {
        let result = p.add(n);
        result.to_a() == 0
    });

    r.record("mul(P, P) == P", p.multiply(&p).to_a() == 1);
    r.record("mul(P, N) == N", p.multiply(&n).to_a() == -1);
    r.record("mul(N, N) == P", n.multiply(&n).to_a() == 1);
    r.record("mul(Z, P) == Z", z.multiply(&p).to_a() == 0);

    r.record("rotate(-1) == 0", n.rotate().to_a() == 0);
    r.record("rotate(0) == 1", z.rotate().to_a() == 1);
    r.record("rotate(1) == -1", p.rotate().to_a() == -1);

    // ── Packed Trit Words ──────────────────────────────────────────
    serial::print_line("  -- Packed Trit Words --");

    let trits = [p, n, z, p, p, n, z, z, p];
    let packed = pack_trits(&trits);
    let unpacked = unpack_trits(packed);
    let mut round_trip_ok = true;
    for i in 0..trits.len() {
        if trits[i].to_a() != unpacked[i].to_a() {
            round_trip_ok = false;
            break;
        }
    }
    r.record("pack/unpack round-trip (9 trits)", round_trip_ok);

    let mut full_word = [z; 27];
    full_word[0] = p;
    full_word[13] = n;
    full_word[26] = p;
    let packed_full = pack_trits(&full_word);
    let unpacked_full = unpack_trits(packed_full);
    r.record("27-trit pack/unpack: pos 0", unpacked_full[0].to_a() == 1);
    r.record("27-trit pack/unpack: pos 13", unpacked_full[13].to_a() == -1);
    r.record("27-trit pack/unpack: pos 26", unpacked_full[26].to_a() == 1);

    // ── Representation Conversion ──────────────────────────────────
    serial::print_line("  -- Representation Bijections --");

    let a_val: i8 = 1;
    let b_val = convert_representation(a_val, Representation::A, Representation::B);
    let c_val = convert_representation(b_val, Representation::B, Representation::C);
    let a_back = convert_representation(c_val, Representation::C, Representation::A);
    r.record("A->B->C->A round trip (+1)", a_back == a_val);

    let a_neg: i8 = -1;
    let b_neg = convert_representation(a_neg, Representation::A, Representation::B);
    let a_neg_back = convert_representation(b_neg, Representation::B, Representation::A);
    r.record("A->B->A round trip (-1)", a_neg_back == a_neg);

    // ── Boot Sequence ──────────────────────────────────────────────
    serial::print_line("  -- Boot Sequence --");

    let mut seq = BootSequence::new(ArchId::X86_64);
    r.record("boot starts at PowerOn", *seq.current_stage() == BootStage::PowerOn);
    r.record("boot not complete initially", !seq.is_complete());

    let mut all_advanced = true;
    for _ in 0..11 {
        if seq.advance().is_err() {
            all_advanced = false;
            break;
        }
    }
    r.record("all 11 boot stages advance", all_advanced);
    r.record("boot sequence complete", seq.is_complete());
    r.record("final stage is Running", *seq.current_stage() == BootStage::Running);
    r.record("11 stages recorded", seq.stages_completed().len() == 11);
    r.record("cannot advance past Running", seq.advance().is_err());

    // ── Boot Config (x86_64) ───────────────────────────────────────
    serial::print_line("  -- x86_64 Boot Config --");

    let params = x86_64_boot_config();
    r.record("arch is X86_64", params.arch_id == ArchId::X86_64);
    r.record("kernel at 0x100000 (1 MiB)",
        params.kernel_physical_base == 0x0010_0000);
    r.record("memory map has entries", params.memory_map.len() >= 2);
    r.record("first region is Usable",
        params.memory_map[0].region_type == MemoryRegionType::Usable);

    // ── Femtosecond Timing ─────────────────────────────────────────
    serial::print_line("  -- Femtosecond Timing --");

    r.record("FS_PER_NS == 1_000_000", FS_PER_NS == 1_000_000);
    r.record("FS_PER_MS == 1e12", FS_PER_MS == 1_000_000_000_000);
    r.record("FS_PER_SECOND == 1e15", FS_PER_SECOND == 1_000_000_000_000_000);

    let ts = FemtosecondTimestamp::new(0);
    r.record("zero timestamp", ts.femtoseconds == 0);
    r.record("zero seconds()", ts.seconds() == 0);

    let ts1s = FemtosecondTimestamp::new(FS_PER_SECOND);
    r.record("1s timestamp: seconds() == 1", ts1s.seconds() == 1);
    r.record("1s timestamp: sub_second_fs() == 0", ts1s.sub_second_fs() == 0);

    let ts_ms = FemtosecondTimestamp::new(FS_PER_SECOND + 500 * FS_PER_MS);
    r.record("1.5s timestamp: ms == 500", ts_ms.milliseconds() == 500);

    let unix_ns = SALVI_EPOCH_NS + 1_000_000_000;
    let ts_from_unix = FemtosecondTimestamp::from_unix_ns(unix_ns);
    r.record("from_unix_ns produces non-zero", ts_from_unix.femtoseconds > 0);

    // ── Phase Encryption ───────────────────────────────────────────
    serial::print_line("  -- Phase Encryption --");

    r.record("HighSecurity: 7 phases",
        EncryptionMode::HighSecurity.phase_count() == 7);
    r.record("Balanced: 5 phases",
        EncryptionMode::Balanced.phase_count() == 5);
    r.record("Performance: 3 phases",
        EncryptionMode::Performance.phase_count() == 3);

    let ratio = EncryptionMode::HighSecurity.split_ratio();
    r.record("HighSecurity uses golden ratio",
        ratio > 0.617 && ratio < 0.619);

    // ── Architecture Detection ─────────────────────────────────────
    serial::print_line("  -- Architecture --");

    let arch = Architecture::detect();
    r.record("architecture detected (not Unknown)", arch != Architecture::Unknown);

    r
}
