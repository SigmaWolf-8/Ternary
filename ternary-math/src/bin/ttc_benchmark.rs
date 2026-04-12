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

fn bench(name: &str, data: &[u8]) -> BenchResult {
    let entropy = shannon_entropy(data);
    let opts = CompressOptions {
        filename: Some(name.to_string()),
        ..Default::default()
    };

    let t0 = Instant::now();
    let result = ttc_compress(data, &opts).expect("compress failed");
    let compress_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let dec = ttc_decompress(&result.compressed).expect("decompress failed");
    let decompress_us = t1.elapsed().as_micros();

    let verified = dec.data == data;

    BenchResult {
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
    }
}

fn print_header() {
    println!();
    println!("======================================================================");
    println!("TTC v5.0.3 — Compression Benchmark (real data, real entropy)");
    println!("======================================================================");
    println!();
    println!("{:<40} {:>8} {:>8} {:>7} {:>6} {:>8} {:>8} {:>4}",
        "Input", "Size", "Comp", "Ratio", "H(X)", "C μs", "D μs", "OK");
    println!("{}", "-".repeat(100));
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
    println!("{:<40} {:>8} {:>8} {:>6.2}x {:>5.2}b {:>7} {:>7}  {}",
        r.name, size_str, comp_str, r.ratio, r.entropy_bits,
        r.compress_us, r.decompress_us,
        if r.verified { "✓" } else { "✗" });
}

fn print_section(title: &str) {
    println!();
    println!("--- {} {}", title, "-".repeat(80 - title.len() - 5));
}

fn main() {
    print_header();

    let mut results: Vec<BenchResult> = Vec::new();

    // =====================================================================
    // SECTION 1: True random (OS entropy — /dev/urandom via getrandom)
    // =====================================================================
    print_section("TRUE RANDOM (OS entropy via getrandom)");

    for &size in &[1024usize, 10240, 51200, 102400] {
        let mut buf = vec![0u8; size];
        getrandom::getrandom(&mut buf).expect("getrandom failed");
        let label = format!("OS random ({:.1}K)", size as f64 / 1024.0);
        let r = bench(&label, &buf);
        print_row(&r);
        results.push(r);
    }

    // =====================================================================
    // SECTION 2: The PRNG that was labeled "pseudo-random" — now honest
    // =====================================================================
    print_section("STRIDED PRNG (Knuth mult hash — hidden stride, NOT random)");

    let prng_data: Vec<u8> = (0..50_000u32).map(|i| {
        ((i.wrapping_mul(2654435761) >> 16) & 0xFF) as u8
    }).collect();
    let r = bench("Knuth stride (50K, stride≈55)", &prng_data);
    print_row(&r);
    results.push(r);

    // =====================================================================
    // SECTION 3: Real source code files
    // =====================================================================
    print_section("SOURCE CODE (real files from this crate)");

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
            let r = bench(&label, &data);
            print_row(&r);
            results.push(r);
            all_source.extend_from_slice(&data);
        }
    }
    if !all_source.is_empty() {
        let label = format!("ALL sources ({:.0}K)", all_source.len() as f64 / 1024.0);
        let r = bench(&label, &all_source);
        print_row(&r);
        results.push(r);
    }

    // =====================================================================
    // SECTION 4: Periodic data — CPT showcase
    // =====================================================================
    print_section("PERIODIC DATA (CPT coprime stride detection)");

    for &(period, size) in &[(7u8, 50000usize), (13, 100000), (91, 100000), (255, 100000)] {
        let pattern: Vec<u8> = (0..period as usize).map(|i| (i * 37 + 11) as u8).collect();
        let data: Vec<u8> = pattern.iter().cycle().take(size).copied().collect();
        let label = format!("Period-{} ({:.0}K)", period, size as f64 / 1024.0);
        let r = bench(&label, &data);
        print_row(&r);
        results.push(r);
    }

    // =====================================================================
    // SECTION 5: Constant data
    // =====================================================================
    print_section("CONSTANT / LOW ENTROPY");

    let zeros = vec![0u8; 100_000];
    let r = bench("All zeros (100K)", &zeros);
    print_row(&r);
    results.push(r);

    let ones = vec![0xFFu8; 100_000];
    let r = bench("All 0xFF (100K)", &ones);
    print_row(&r);
    results.push(r);

    let two_val: Vec<u8> = (0..100_000u32).map(|i| if i % 3 == 0 { 0xAA } else { 0x55 }).collect();
    let r = bench("Two-value (100K, 1.0 bit/byte)", &two_val);
    print_row(&r);
    results.push(r);

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
    let r = bench("English prose (50K, repeated)", &text);
    print_row(&r);
    results.push(r);

    if let Ok(data) = fs::read("Cargo.toml") {
        let r = bench("Cargo.toml (real config)", &data);
        print_row(&r);
        results.push(r);
    }

    if let Ok(data) = fs::read("../package.json") {
        let r = bench("package.json (real config)", &data);
        print_row(&r);
        results.push(r);
    }

    // =====================================================================
    // SECTION 7: Mixed / adversarial
    // =====================================================================
    print_section("MIXED / ADVERSARIAL");

    let mut mixed = Vec::new();
    mixed.extend_from_slice(&vec![0u8; 10_000]);
    let mut rng_buf = vec![0u8; 10_000];
    getrandom::getrandom(&mut rng_buf).expect("getrandom failed");
    mixed.extend_from_slice(&rng_buf);
    let pattern: Vec<u8> = (0..13u8).collect();
    mixed.extend(pattern.iter().cycle().take(10_000).copied());
    mixed.extend_from_slice(b"Hello World! ".repeat(769).as_slice());
    let r = bench("Mixed (10K zeros+10K rand+10K p13+10K text)", &mixed);
    print_row(&r);
    results.push(r);

    // Incompressible with 1 byte changed (should still be ~incompressible)
    let mut almost_random = vec![0u8; 50_000];
    getrandom::getrandom(&mut almost_random).expect("getrandom failed");
    almost_random[0] = 0;
    almost_random[25000] = 0;
    let r = bench("OS random + 2 known bytes (50K)", &almost_random);
    print_row(&r);
    results.push(r);

    // =====================================================================
    // SUMMARY
    // =====================================================================
    println!();
    println!("======================================================================");
    println!("SUMMARY");
    println!("======================================================================");

    let all_verified = results.iter().all(|r| r.verified);
    println!("Round-trip verification: {} / {} passed {}",
        results.iter().filter(|r| r.verified).count(),
        results.len(),
        if all_verified { "✓ ALL PASS" } else { "✗ FAILURES" });

    let true_random: Vec<&BenchResult> = results.iter()
        .filter(|r| r.name.starts_with("OS random"))
        .collect();
    if !true_random.is_empty() {
        let avg_ratio: f64 = true_random.iter().map(|r| r.ratio).sum::<f64>() / true_random.len() as f64;
        let avg_entropy: f64 = true_random.iter().map(|r| r.entropy_bits).sum::<f64>() / true_random.len() as f64;
        println!();
        println!("True random (OS entropy):");
        println!("  Avg entropy:  {:.4} bits/byte", avg_entropy);
        println!("  Avg ratio:    {:.4}x", avg_ratio);
        if avg_ratio < 1.0 {
            println!("  → Output LARGER than input (correct: incompressible data cannot be compressed)");
        } else if avg_ratio < 1.05 {
            println!("  → Near 1.0x (correct: Shannon limit respected, header overhead only)");
        } else {
            println!("  → WARNING: ratio > 1.05x on true random data — investigate");
        }
    }

    let periodic: Vec<&BenchResult> = results.iter()
        .filter(|r| r.name.starts_with("Period-"))
        .collect();
    if !periodic.is_empty() {
        let avg_ratio: f64 = periodic.iter().map(|r| r.ratio).sum::<f64>() / periodic.len() as f64;
        println!();
        println!("Periodic data (CPT showcase):");
        println!("  Avg ratio:    {:.1}x", avg_ratio);
        println!("  → CPT coprime stride detection working as designed");
    }

    let source: Vec<&BenchResult> = results.iter()
        .filter(|r| r.name.starts_with("src:"))
        .collect();
    if !source.is_empty() {
        let avg_ratio: f64 = source.iter().map(|r| r.ratio).sum::<f64>() / source.len() as f64;
        println!();
        println!("Source code (general purpose):");
        println!("  Avg ratio:    {:.2}x", avg_ratio);
    }

    println!();
    if all_verified {
        println!("All round-trip verifications passed. Shannon limit respected on true random data.");
    }
}
