// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
// Author: RSalvi@Salvigroup.com
//
// TTC v4.1 Comprehensive Benchmark Suite
// Industry-standard test patterns with speed/throughput metrics

use std::time::{Duration, Instant};

use ternary_math::ttc::{
    ttc_compress, ttc_decompress, CompressOptions, CompressionMode, crc32, compute_entropy,
};

const WARMUP_ITERS: usize = 1;
const MIN_ITERS: usize = 3;
const MIN_BENCH_TIME_MS: u128 = 100;

struct BenchResult {
    label: String,
    size: usize,
    level: u8,
    comp_us: f64,
    dec_us: f64,
    comp_throughput_mbs: f64,
    dec_throughput_mbs: f64,
    ratio: f64,
    saved_pct: f64,
    compressed_size: usize,
    mode_name: String,
    chunks: usize,
}

fn bench_compress_decompress(
    label: &str, data: &[u8], level: u8, mode: CompressionMode, independent: bool,
) -> BenchResult {
    let opts = CompressOptions {
        mode, level, independent_chunks: independent,
        compute_fibonacci: false, image_width: None, filename: None,
    };

    for _ in 0..WARMUP_ITERS {
        let _ = ttc_compress(data, &opts);
    }

    let mut comp_elapsed = Duration::ZERO;
    let mut comp_iters = 0usize;
    let result = loop {
        let t0 = Instant::now();
        let r = ttc_compress(data, &opts).unwrap();
        comp_elapsed += t0.elapsed();
        comp_iters += 1;
        if comp_iters >= MIN_ITERS && comp_elapsed.as_millis() >= MIN_BENCH_TIME_MS {
            break r;
        }
    };
    let comp_us = comp_elapsed.as_micros() as f64 / comp_iters as f64;

    for _ in 0..WARMUP_ITERS {
        let _ = ttc_decompress(&result.compressed);
    }

    let mut dec_elapsed = Duration::ZERO;
    let mut dec_iters = 0usize;
    loop {
        let t0 = Instant::now();
        let dec = ttc_decompress(&result.compressed).unwrap();
        dec_elapsed += t0.elapsed();
        dec_iters += 1;
        assert_eq!(dec.data.len(), data.len(), "Round-trip size mismatch for '{label}'");
        assert_eq!(dec.data, data, "Round-trip data mismatch for '{label}'");
        if dec_iters >= MIN_ITERS && dec_elapsed.as_millis() >= MIN_BENCH_TIME_MS {
            break;
        }
    }
    let dec_us = dec_elapsed.as_micros() as f64 / dec_iters as f64;

    let size = data.len();
    let comp_throughput = size as f64 / comp_us; // bytes/µs = MB/s
    let dec_throughput = size as f64 / dec_us;
    let ratio = size as f64 / result.compressed_size as f64;
    let saved = (1.0 - result.compressed_size as f64 / size as f64) * 100.0;
    let mode_name = match result.chunks.first().map(|c| c.mode) {
        Some(0) => "Stored", Some(1) => "Comp", Some(2) => "TernEnh", Some(3) => "rANS/3", _ => "?"
    };

    BenchResult {
        label: label.to_string(), size, level, comp_us, dec_us,
        comp_throughput_mbs: comp_throughput, dec_throughput_mbs: dec_throughput,
        ratio, saved_pct: saved, compressed_size: result.compressed_size as usize,
        mode_name: mode_name.to_string(), chunks: result.chunks.len(),
    }
}

fn generate_calgary_text(size: usize) -> Vec<u8> {
    let seed_text = b"The quick brown fox jumps over the lazy dog. \
        In the beginning was the Word, and the Word was with God. \
        To be, or not to be, that is the question. \
        Call me Ishmael. Some years ago never mind how long precisely. \
        It was the best of times, it was the worst of times. \
        All happy families are alike; each unhappy family is unhappy in its own way. \
        It is a truth universally acknowledged, that a single man in possession of a good fortune. \
        Mr. and Mrs. Dursley, of number four, Privet Drive, were proud to say that they were perfectly normal. ";
    (0..size).map(|i| seed_text[i % seed_text.len()]).collect()
}

fn generate_silesia_binary(size: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(size);
    let mut s = 0xDEADBEEFu32;
    for i in 0..size {
        s = s.wrapping_mul(1103515245).wrapping_add(12345);
        let byte = ((s >> 16) & 0xFF) as u8;
        if i % 7 == 0 { v.push(0); }
        else if i % 13 == 0 { v.push(byte & 0x0F); }
        else { v.push(byte); }
    }
    v
}

fn generate_structured_log(size: usize) -> Vec<u8> {
    let templates = [
        "2026-03-16T10:30:{ss}.{ms}Z INFO  server: Request processed in {n}ms path=/api/v1/compress\n",
        "2026-03-16T10:30:{ss}.{ms}Z DEBUG cache: Cache hit key=session_{id} ttl=3600\n",
        "2026-03-16T10:30:{ss}.{ms}Z WARN  rate_limiter: Rate limit approaching threshold=0.85 client={ip}\n",
        "2026-03-16T10:30:{ss}.{ms}Z INFO  compression: TTC level=5 ratio=3.42x chunks=4 mode=rANS/3\n",
        "2026-03-16T10:30:{ss}.{ms}Z ERROR tunnel: Handshake timeout peer=cube-{id} dimension=7 retry=3\n",
    ];
    let mut out = Vec::with_capacity(size);
    let mut i = 0u32;
    while out.len() < size {
        let tmpl = templates[(i as usize) % templates.len()];
        let line = tmpl
            .replace("{ss}", &format!("{:02}", i % 60))
            .replace("{ms}", &format!("{:03}", (i * 137) % 1000))
            .replace("{n}", &format!("{}", 2 + (i * 7) % 500))
            .replace("{id}", &format!("{:04x}", (i * 2654435761u32) & 0xFFFF))
            .replace("{ip}", &format!("10.{}.{}.{}", (i/256)%256, (i/16)%256, i%256));
        out.extend_from_slice(line.as_bytes());
        i += 1;
    }
    out.truncate(size);
    out
}

fn generate_source_code(size: usize) -> Vec<u8> {
    let code = b"pub fn compress_chunk(\n    chunk: &[u8], history: &[u8], idx: usize,\n    cfg: &LevelConfig, mode: CompressionMode,\n    independent: bool, tc: &TritCostTables,\n) -> ChunkResult {\n    let p1 = phase1_analyze(chunk, idx, cfg, mode);\n    phase2_compress(chunk, &p1, history, cfg, independent, tc, DomainTransform::NONE)\n}\n\nfn tokenize_greedy_lazy(data: &[u8], hist_off: usize, cfg: &LevelConfig) -> Vec<Token> {\n    let mut eng = Lz77Engine::new(cfg);\n    let mut tokens = Vec::new();\n    for j in 0..hist_off.min(data.len()) { eng.update(data, j); }\n    let mut i = hist_off;\n    while i < data.len() {\n        if let Some((dist, len)) = eng.find_best_match(data, i) {\n            tokens.push(Token::Match { dist, length: len });\n            for k in 0..len { eng.update(data, i+k); }\n            i += len; continue;\n        }\n        tokens.push(Token::Literal(data[i]));\n        eng.update(data, i); i += 1;\n    }\n    tokens\n}\n\n";
    (0..size).map(|i| code[i % code.len()]).collect()
}

fn generate_genomic(size: usize) -> Vec<u8> {
    let bases = b"ACGTACGTAAGGCCTTACGTACGTTTAACCGGACGTACGTACGTACGT";
    let mut out = Vec::with_capacity(size);
    let mut s = 42u32;
    for i in 0..size {
        s = s.wrapping_mul(48271).wrapping_add(1);
        if (s >> 8) % 20 == 0 {
            out.push(b'\n');
        } else {
            out.push(bases[(i + (s as usize >> 16)) % bases.len()]);
        }
    }
    out
}

fn generate_constant(size: usize) -> Vec<u8> { vec![0xAAu8; size] }

fn generate_random(size: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(size);
    let mut s = 0x5EED_1234u64;
    for _ in 0..size {
        s ^= s << 13; s ^= s >> 7; s ^= s << 17;
        v.push((s & 0xFF) as u8);
    }
    v
}

fn generate_json(size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(b"[\n");
    let mut i = 0u32;
    while out.len() < size.saturating_sub(4) {
        let entry = format!(
            "  {{\"id\": {}, \"name\": \"user_{:04x}\", \"email\": \"user{}@plenum.net\", \"role\": \"{}\", \"active\": {}, \"score\": {:.2}}},\n",
            i, i, i, ["admin", "user", "viewer", "editor"][(i as usize) % 4],
            i % 3 != 0, (i as f64 * 3.14159) % 100.0
        );
        out.extend_from_slice(entry.as_bytes());
        i += 1;
    }
    out.extend_from_slice(b"]\n");
    out.truncate(size);
    out
}

fn generate_csv(size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(b"timestamp,node_id,cpu_pct,mem_mb,packets_in,packets_out,latency_us\n");
    let mut ts = 1710600000u64;
    let mut i = 0u32;
    while out.len() < size {
        let line = format!(
            "{},{},{:.1},{},{},{},{}\n",
            ts, format!("cube-{:04x}", i % 4096),
            20.0 + (i as f64 * 1.7) % 80.0, 512 + (i * 37) % 3584,
            1000 + (i * 131) % 50000, 800 + (i * 97) % 45000,
            100 + (i * 23) % 9900
        );
        out.extend_from_slice(line.as_bytes());
        ts += 1; i += 1;
    }
    out.truncate(size);
    out
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.0} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}

fn print_section_header(title: &str) {
    eprintln!("\n### {}\n", title);
    eprintln!("| {:<14} | {:<6} | {:>8} | {:>8} | {:>9} | {:>9} | {:>7} | {:>6} | {:<7} | {:>6} |",
        "Dataset", "Size", "Comp µs", "Dec µs", "Comp MB/s", "Dec MB/s", "Ratio", "Saved%", "Mode", "Chunks");
    eprintln!("|{}", "-".repeat(115));
}

fn print_result(r: &BenchResult) {
    eprintln!("| {:<14} | {:<6} | {:>8.0} | {:>8.0} | {:>9.2} | {:>9.2} | {:>7.2}x | {:>5.1}% | {:<7} | {:>6} |",
        r.label, format_size(r.size), r.comp_us, r.dec_us,
        r.comp_throughput_mbs, r.dec_throughput_mbs,
        r.ratio, r.saved_pct, r.mode_name, r.chunks);
}

fn main() {
    eprintln!("# TTC v4.1 Comprehensive Benchmark Report");
    eprintln!();
    eprintln!("**Date:** {}", "2026-03-16");
    eprintln!("**Engine:** TM-2026-017 TTC v4.1 — ternary rANS + hybrid Rice, OnceLock tables");
    eprintln!("**Platform:** Replit Linux x86_64, Rust release build");
    eprintln!("**Protocol:** Tribonacci Ternary Compression v2.0 (wire format)");
    eprintln!("**Iterations:** min {} per measurement, min {}ms wall-clock", MIN_ITERS, MIN_BENCH_TIME_MS);
    eprintln!();

    let mut all_results: Vec<BenchResult> = Vec::new();

    // ─── Section 1: Calgary Corpus-style text ────────────────────────────
    {
        print_section_header("1. English Text (Calgary Corpus-style)");
        for &(size, level) in &[
            (1024, 1), (4096, 2), (16384, 3), (65536, 4),
            (262144, 5), (1048576, 6),
        ] {
            let data = generate_calgary_text(size);
            let r = bench_compress_decompress(
                &format!("text-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 2: Structured data (JSON) ───────────────────────────────
    {
        print_section_header("2. Structured Data (JSON)");
        for &(size, level) in &[
            (4096, 2), (16384, 3), (65536, 4), (262144, 5), (1048576, 6),
        ] {
            let data = generate_json(size);
            let r = bench_compress_decompress(
                &format!("json-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 3: Log data ─────────────────────────────────────────────
    {
        print_section_header("3. Server Logs (timestamped, structured)");
        for &(size, level) in &[
            (4096, 2), (16384, 3), (65536, 4), (262144, 5), (1048576, 6),
        ] {
            let data = generate_structured_log(size);
            let r = bench_compress_decompress(
                &format!("log-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 4: Source code ───────────────────────────────────────────
    {
        print_section_header("4. Source Code (Rust-like)");
        for &(size, level) in &[
            (4096, 2), (16384, 3), (65536, 4), (262144, 5), (1048576, 6),
        ] {
            let data = generate_source_code(size);
            let r = bench_compress_decompress(
                &format!("source-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 5: Genomic data ─────────────────────────────────────────
    {
        print_section_header("5. Genomic Sequences (DNA ACGT)");
        for &(size, level) in &[
            (4096, 2), (16384, 3), (65536, 4), (262144, 5), (1048576, 6),
        ] {
            let data = generate_genomic(size);
            let r = bench_compress_decompress(
                &format!("genomic-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 6: CSV / time-series ────────────────────────────────────
    {
        print_section_header("6. CSV Time-Series Data");
        for &(size, level) in &[
            (4096, 2), (16384, 3), (65536, 4), (262144, 5), (1048576, 6),
        ] {
            let data = generate_csv(size);
            let r = bench_compress_decompress(
                &format!("csv-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 7: Binary/mixed ─────────────────────────────────────────
    {
        print_section_header("7. Binary Data (mixed entropy)");
        for &(size, level) in &[
            (4096, 2), (65536, 4), (262144, 5), (1048576, 6),
        ] {
            let data = generate_silesia_binary(size);
            let r = bench_compress_decompress(
                &format!("binary-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 8: Edge cases ───────────────────────────────────────────
    {
        print_section_header("8. Edge Cases");
        let cases: Vec<(&str, Vec<u8>, u8, CompressionMode)> = vec![
            ("constant-1K", generate_constant(1024), 1, CompressionMode::Basic),
            ("constant-64K", generate_constant(65536), 4, CompressionMode::Basic),
            ("constant-1M", generate_constant(1048576), 6, CompressionMode::Basic),
            ("random-1K", generate_random(1024), 1, CompressionMode::Basic),
            ("random-64K", generate_random(65536), 4, CompressionMode::Basic),
            ("random-1M", generate_random(1048576), 6, CompressionMode::Basic),
        ];
        for (name, data, level, mode) in &cases {
            let r = bench_compress_decompress(name, data, *level, *mode, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Section 9: Level scaling (fixed 256 KB text) ────────────────────
    {
        print_section_header("9. Level Scaling (256 KB English text)");
        let data = generate_calgary_text(262144);
        for level in 1..=9u8 {
            let r = bench_compress_decompress(
                &format!("256K-L{}", level), &data, level, CompressionMode::Basic, true);
            print_result(&r);
            all_results.push(r);
        }
    }

    // ─── Summary statistics ──────────────────────────────────────────────
    eprintln!("\n---\n");
    eprintln!("## Summary Statistics\n");

    let text_results: Vec<&BenchResult> = all_results.iter().filter(|r| r.label.starts_with("text-")).collect();
    if !text_results.is_empty() {
        let avg_comp: f64 = text_results.iter().map(|r| r.comp_throughput_mbs).sum::<f64>() / text_results.len() as f64;
        let avg_dec: f64 = text_results.iter().map(|r| r.dec_throughput_mbs).sum::<f64>() / text_results.len() as f64;
        let avg_ratio: f64 = text_results.iter().map(|r| r.ratio).sum::<f64>() / text_results.len() as f64;
        eprintln!("**Text corpus avg:** {:.2} MB/s compress, {:.2} MB/s decompress, {:.2}x ratio", avg_comp, avg_dec, avg_ratio);
    }

    let all_comp: Vec<f64> = all_results.iter().map(|r| r.comp_throughput_mbs).collect();
    let all_dec: Vec<f64> = all_results.iter().map(|r| r.dec_throughput_mbs).collect();
    let peak_comp = all_comp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let peak_dec = all_dec.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let best_ratio = all_results.iter().map(|r| r.ratio).fold(f64::NEG_INFINITY, f64::max);

    eprintln!("**Peak compress throughput:** {:.2} MB/s", peak_comp);
    eprintln!("**Peak decompress throughput:** {:.2} MB/s", peak_dec);
    eprintln!("**Best compression ratio:** {:.2}x", best_ratio);

    let total_input: usize = all_results.iter().map(|r| r.size).sum();
    let total_output: usize = all_results.iter().map(|r| r.compressed_size).sum();
    eprintln!("**Aggregate:** {:.2} MB input → {:.2} MB compressed ({:.2}x overall)",
        total_input as f64 / 1_048_576.0, total_output as f64 / 1_048_576.0,
        total_input as f64 / total_output as f64);

    eprintln!("\n**CRC32 check (SSE4.2 dispatch):** 0x{:08X}", crc32(b"123456789"));
    eprintln!("**All round-trips verified.** Every decompressed output matched original input byte-for-byte.");
    eprintln!();
}
