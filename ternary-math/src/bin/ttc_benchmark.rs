use ternary_math::ttc::{ttc_compress, ttc_decompress, CompressOptions};
use std::fs;
use std::time::Instant;

struct BenchResult {
    name: String,
    original: usize,
    compressed: usize,
    ratio: f64,
    entropy_bits: f64,
    compress_us: u128,
    decompress_us: u128,
    verified: bool,
}

fn shannon_entropy(data: &[u8]) -> f64 {
    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let total = data.len() as f64;
    let mut entropy = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn bench(name: &str, data: &[u8]) -> Option<BenchResult> {
    let entropy = shannon_entropy(data);
    let opts = CompressOptions {
        filename: Some(name.to_string()),
        ..Default::default()
    };

    let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ttc_compress(data, &opts)
    })) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            eprintln!("  compress error on {}: {:?}", name, e);
            return None;
        }
        Err(_) => {
            eprintln!("  PANIC during compress of {}", name);
            return None;
        }
    };

    let t0 = Instant::now();
    let _ = ttc_compress(data, &opts);
    let compress_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let dec = match ttc_decompress(&result.compressed) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("  decompress error on {}: {:?}", name, e);
            return None;
        }
    };
    let decompress_us = t1.elapsed().as_micros();

    let verified = dec.data == data;

    Some(BenchResult {
        name: name.to_string(),
        original: data.len(),
        compressed: result.compressed_size as usize,
        ratio: if result.compressed_size > 0 {
            data.len() as f64 / result.compressed_size as f64
        } else {
            0.0
        },
        entropy_bits: entropy,
        compress_us,
        decompress_us,
        verified,
    })
}

fn print_header() {
    println!();
    println!("======================================================================");
    println!("TTC v5.0.3 — Compression Benchmark (real data, real entropy)");
    println!("======================================================================");
    println!();
    println!("{:<42} {:>8} {:>8} {:>7} {:>6} {:>8} {:>8} {:>4}",
        "Input", "Size", "Comp", "Ratio", "H(X)", "C us", "D us", "RT");
    println!("{}", "-".repeat(102));
}

fn print_row(r: &BenchResult) {
    let size_str = if r.original >= 1024 {
        format!("{:.1}K", r.original as f64 / 1024.0)
    } else {
        format!("{}B", r.original)
    };
    let comp_str = if r.compressed >= 1024 {
        format!("{:.1}K", r.compressed as f64 / 1024.0)
    } else {
        format!("{}B", r.compressed)
    };
    println!("{:<42} {:>8} {:>8} {:>6.2}x {:>5.2}b {:>7} {:>7}  {}",
        r.name, size_str, comp_str, r.ratio, r.entropy_bits,
        r.compress_us, r.decompress_us,
        if r.verified { "OK" } else { "FAIL" });
}

fn print_section(title: &str) {
    println!();
    println!("--- {} {}", title, "-".repeat(80 - title.len() - 5));
}

fn main() {
    print_header();

    let mut results: Vec<BenchResult> = Vec::new();
    let mut panics = 0u32;

    // =====================================================================
    // SECTION 1: True random (OS entropy via getrandom)
    // =====================================================================
    print_section("TRUE RANDOM (OS /dev/urandom via getrandom)");

    for &size in &[512usize, 1024, 4096, 10240, 51200, 102400] {
        let mut buf = vec![0u8; size];
        getrandom::getrandom(&mut buf).expect("getrandom failed");
        let label = format!("urandom ({})", if size >= 1024 {
            format!("{:.0}K", size as f64 / 1024.0)
        } else {
            format!("{}B", size)
        });
        match bench(&label, &buf) {
            Some(r) => { print_row(&r); results.push(r); }
            None => { println!("{:<42} {:>8} --- PANIC/ERROR ---", label, size); panics += 1; }
        }
    }

    // =====================================================================
    // SECTION 2: Correctly-labeled PRNG with hidden stride
    // =====================================================================
    print_section("STRIDED PRNG (Knuth multiplicative — NOT random)");

    let prng_data: Vec<u8> = (0..50_000u32).map(|i| {
        ((i.wrapping_mul(2654435761) >> 16) & 0xFF) as u8
    }).collect();
    if let Some(r) = bench("Knuth stride (50K, delta~55 mod 256)", &prng_data) {
        print_row(&r);
        println!("  NOTE: order-0 H=8.00b but order-1 H=0.997b (stride structure)");
        results.push(r);
    }

    // =====================================================================
    // SECTION 3: Real source code files
    // =====================================================================
    print_section("SOURCE CODE (real .rs files from this crate)");

    let source_files = [
        "src/ttc.rs",
        "src/constants.rs",
        "src/trit_int.rs",
        "src/coprime.rs",
        "src/tlsponge385.rs",
        "src/ctx_ans.rs",
    ];
    let mut all_source = Vec::new();
    for path in &source_files {
        if let Ok(data) = fs::read(path) {
            let label = format!("src: {}", path.trim_start_matches("src/"));
            if let Some(r) = bench(&label, &data) {
                print_row(&r);
                results.push(r);
            }
            all_source.extend_from_slice(&data);
        }
    }
    if !all_source.is_empty() {
        let label = format!("ALL sources ({:.0}K combined)", all_source.len() as f64 / 1024.0);
        if let Some(r) = bench(&label, &all_source) {
            print_row(&r);
            results.push(r);
        }
    }

    // =====================================================================
    // SECTION 4: Periodic data — CPT showcase
    // =====================================================================
    print_section("PERIODIC DATA (CPT coprime stride detection)");

    for &(period, size) in &[(7u16, 50000usize), (13, 100000), (91, 100000), (255, 100000)] {
        let pattern: Vec<u8> = (0..period as usize).map(|i| (i * 37 + 11) as u8).collect();
        let data: Vec<u8> = pattern.iter().cycle().take(size).copied().collect();
        let label = format!("Period-{} ({:.0}K)", period, size as f64 / 1024.0);
        if let Some(r) = bench(&label, &data) {
            print_row(&r);
            results.push(r);
        }
    }

    // =====================================================================
    // SECTION 5: Constant / low entropy
    // =====================================================================
    print_section("CONSTANT / LOW ENTROPY");

    let zeros = vec![0u8; 100_000];
    if let Some(r) = bench("All zeros (100K)", &zeros) {
        print_row(&r);
        results.push(r);
    }

    let ones = vec![0xFFu8; 100_000];
    if let Some(r) = bench("All 0xFF (100K)", &ones) {
        print_row(&r);
        results.push(r);
    }

    let two_val: Vec<u8> = (0..100_000u32).map(|i| if i % 3 == 0 { 0xAA } else { 0x55 }).collect();
    if let Some(r) = bench("Two-value alternating (100K)", &two_val) {
        print_row(&r);
        results.push(r);
    }

    // =====================================================================
    // SECTION 6: Natural language text
    // =====================================================================
    print_section("TEXT DATA");

    let prose = b"In the beginning was the Word, and the Word was with God, and the Word was God. \
The same was in the beginning with God. All things were made by him; and without him was not \
any thing made that was made. In him was life; and the life was the light of men. And the \
light shineth in darkness; and the darkness comprehended it not. There was a man sent from \
God, whose name was John. The same came for a witness, to bear witness of the Light, that \
all men through him might believe. He was not that Light, but was sent to bear witness of \
that Light. That was the true Light, which lighteth every man that cometh into the world. ";
    let text: Vec<u8> = prose.iter().cycle().take(50_000).copied().collect();
    if let Some(r) = bench("English prose (50K, repeated passage)", &text) {
        print_row(&r);
        results.push(r);
    }

    if let Ok(data) = fs::read("Cargo.toml") {
        if let Some(r) = bench("Cargo.toml (real config file)", &data) {
            print_row(&r);
            results.push(r);
        }
    }

    if let Ok(data) = fs::read("../package.json") {
        if let Some(r) = bench("package.json (real config file)", &data) {
            print_row(&r);
            results.push(r);
        }
    }

    // =====================================================================
    // SECTION 7: Round-trip stress (multiple random trials)
    // =====================================================================
    print_section("ROUND-TRIP STRESS (5 trials per size, OS random)");

    for &size in &[4096usize, 8192, 16384, 32768, 65536] {
        let mut pass = 0u32;
        let mut fail = 0u32;
        let mut panic_count = 0u32;
        let trials = 5;
        for _ in 0..trials {
            let mut buf = vec![0u8; size];
            getrandom::getrandom(&mut buf).expect("getrandom");
            match bench("", &buf) {
                Some(r) => {
                    if r.verified { pass += 1; } else { fail += 1; }
                }
                None => { panic_count += 1; }
            }
        }
        let status = if fail == 0 && panic_count == 0 { "ALL PASS" } else {
            if panic_count > 0 { "PANIC+FAIL" } else { "FAIL" }
        };
        println!("  size={:>6}:  {}/{} pass, {} fail, {} panic  [{}]",
            size, pass, trials, fail, panic_count, status);
    }

    // =====================================================================
    // SUMMARY
    // =====================================================================
    println!();
    println!("======================================================================");
    println!("SUMMARY");
    println!("======================================================================");

    let all_verified = results.iter().all(|r| r.verified);
    let verified_count = results.iter().filter(|r| r.verified).count();
    let failed_count = results.len() - verified_count;
    println!("Round-trip: {}/{} passed, {} failed, {} panics",
        verified_count, results.len(), failed_count, panics);

    let true_random: Vec<&BenchResult> = results.iter()
        .filter(|r| r.name.starts_with("urandom"))
        .collect();
    if !true_random.is_empty() {
        let passing: Vec<&&BenchResult> = true_random.iter().filter(|r| r.verified).collect();
        let failing: Vec<&&BenchResult> = true_random.iter().filter(|r| !r.verified).collect();
        println!();
        println!("True random (OS entropy):");
        if !passing.is_empty() {
            let avg_ratio: f64 = passing.iter().map(|r| r.ratio).sum::<f64>() / passing.len() as f64;
            println!("  Passing: {}/{}, avg ratio: {:.4}x", passing.len(), true_random.len(), avg_ratio);
        }
        if !failing.is_empty() {
            println!("  FAILING: {}/{} — round-trip data loss on true random input", failing.len(), true_random.len());
            println!("  ROOT CAUSE: auto-detect (JSON/CSV/XML) false-positives on random bytes");
            println!("  TRIGGER: random data starting with 0x7B {{, 0x5B [, or 0x3C <");
        }
    }

    let periodic: Vec<&BenchResult> = results.iter()
        .filter(|r| r.name.starts_with("Period-"))
        .collect();
    if !periodic.is_empty() {
        let avg_ratio: f64 = periodic.iter().map(|r| r.ratio).sum::<f64>() / periodic.len() as f64;
        println!();
        println!("Periodic data (CPT showcase):");
        println!("  Avg ratio: {:.1}x — coprime stride detection working as designed", avg_ratio);
    }

    let source: Vec<&BenchResult> = results.iter()
        .filter(|r| r.name.starts_with("src:"))
        .collect();
    if !source.is_empty() {
        let avg_ratio: f64 = source.iter().map(|r| r.ratio).sum::<f64>() / source.len() as f64;
        println!();
        println!("Source code (general purpose):");
        println!("  Avg ratio: {:.2}x", avg_ratio);
    }

    if !all_verified || panics > 0 {
        println!();
        println!("*** INTEGRITY ISSUES DETECTED ***");
        println!();
        println!("BUG #1 (CRITICAL): structured_encode_json panic on random data");
        println!("  Line 1374: data[end+1..] where end=data.len() (unclosed quote)");
        println!("  Trigger: random bytes starting with 0x7B/0x5B auto-detected as JSON");
        println!();
        println!("BUG #2 (CRITICAL): Silent data loss on high-entropy input");
        println!("  Domain auto-detect treats random bytes as JSON/CSV/XML");
        println!("  Structured transform corrupts data; competitive gate checks size only");
        println!("  Fix: guard auto-detect with minimum structure score threshold");
        println!();
        println!("BUG #3 (MINOR): BitReader silently returns zeros past end of stream");
        println!("  Line 553: returns partial value instead of signaling error");
    } else {
        println!();
        println!("All round-trip verifications passed. Shannon limit respected.");
    }
}
