use std::time::Instant;

fn main() {
    for _ in 0..100 {
        let _ = ternary_math::tlsponge385::derive_key(b"WARMUP", b"warmup-material", 48);
    }

    // ═══════════════════════════════════════════════════════════════
    // A/B Test 1: derive_key scalar — old baseline was 4.088µs
    // ═══════════════════════════════════════════════════════════════
    let iters = 10_000u64;
    let start = Instant::now();
    for i in 0..iters {
        let _ = std::hint::black_box(
            ternary_math::tlsponge385::derive_key(b"AB-TEST", &(i as u32).to_le_bytes(), 48)
        );
    }
    let elapsed = start.elapsed();
    let per_call = elapsed.as_nanos() as f64 / iters as f64;
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  A/B: tlsponge385::derive_key                          │");
    println!("│  {} iterations in {:.3}ms                       │", iters, elapsed.as_secs_f64() * 1000.0);
    println!("│  Per call: {:.0} ns ({:.3} µs)                         │", per_call, per_call / 1000.0);
    println!("│  Old sponge::derive_key: 4088 ns (4.088 µs)            │");
    let speedup = 4088.0 / per_call;
    if per_call < 4088.0 {
        println!("│  ✓ FASTER by {:.1}× ({:.0} ns saved)                     │", speedup, 4088.0 - per_call);
    } else {
        println!("│  ✗ SLOWER by {:.1}× ({:.0} ns added)                     │", 1.0/speedup, per_call - 4088.0);
    }
    println!("└─────────────────────────────────────────────────────────┘");

    // ═══════════════════════════════════════════════════════════════
    // A/B Test 2: derive_key_batch (heartbeat_26) — 26 parallel keys
    // ═══════════════════════════════════════════════════════════════
    let domains: Vec<Vec<u8>> = (0..26u8).map(|i| {
        let mut d = b"PlenumNET-HB-HMAC-".to_vec();
        d.push(i);
        d
    }).collect();
    let materials: Vec<Vec<u8>> = (0..26u8).map(|i| {
        let mut m = b"key-material-for-heartbeat-".to_vec();
        m.push(i);
        m
    }).collect();
    let domain_refs: Vec<&[u8]> = domains.iter().map(|d| d.as_slice()).collect();
    let mat_refs: Vec<&[u8]> = materials.iter().map(|m| m.as_slice()).collect();

    for _ in 0..50 {
        let _ = ternary_math::tlsponge385::derive_key_batch(&domain_refs, &mat_refs, 48);
    }

    let batch_iters = 5_000u64;
    let start = Instant::now();
    for _ in 0..batch_iters {
        let _ = std::hint::black_box(
            ternary_math::tlsponge385::derive_key_batch(&domain_refs, &mat_refs, 48)
        );
    }
    let elapsed = start.elapsed();
    let per_batch = elapsed.as_nanos() as f64 / batch_iters as f64;
    let per_key = per_batch / 26.0;
    let scalar_26 = per_call * 26.0;
    println!();
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  derive_key_batch (heartbeat_26 = 26 parallel keys)    │");
    println!("│  {} iterations in {:.3}ms                       │", batch_iters, elapsed.as_secs_f64() * 1000.0);
    println!("│  Per batch (26 keys): {:.0} ns ({:.3} µs)              │", per_batch, per_batch / 1000.0);
    println!("│  Per key (amortized): {:.0} ns ({:.3} µs)              │", per_key, per_key / 1000.0);
    println!("│  Scalar 26× would be: {:.0} ns ({:.3} µs)             │", scalar_26, scalar_26 / 1000.0);
    let batch_speedup = scalar_26 / per_batch;
    println!("│  Batch vs scalar: {:.2}× throughput                    │", batch_speedup);
    let old_hb26 = 4088.0 * 26.0;
    let vs_old = old_hb26 / per_batch;
    println!("│  Batch vs old scalar×26: {:.2}×                        │", vs_old);
    println!("└─────────────────────────────────────────────────────────┘");

    // ═══════════════════════════════════════════════════════════════
    // A/B Test 3: hash_hex — sanity
    // ═══════════════════════════════════════════════════════════════
    let start = Instant::now();
    for i in 0..iters {
        let _ = std::hint::black_box(
            ternary_math::tlsponge385::hash_hex(&(i as u64).to_le_bytes())
        );
    }
    let elapsed = start.elapsed();
    let per_hash = elapsed.as_nanos() as f64 / iters as f64;
    println!();
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  hash_hex                                              │");
    println!("│  Per call: {:.0} ns ({:.3} µs)                         │", per_hash, per_hash / 1000.0);
    println!("└─────────────────────────────────────────────────────────┘");

    // ═══════════════════════════════════════════════════════════════
    // Verdict
    // ═══════════════════════════════════════════════════════════════
    println!();
    if per_call < 4088.0 && per_batch < scalar_26 {
        println!("══════════════════════════════════════════════════════════");
        println!("  VERDICT: SPONGE IS FIXED");
        println!("  derive_key: {:.3}µs (was 4.088µs)", per_call / 1000.0);
        println!("  heartbeat_26 batch: {:.3}µs (scalar×26 would be {:.3}µs)", per_batch / 1000.0, scalar_26 / 1000.0);
        println!("══════════════════════════════════════════════════════════");
    } else {
        println!("══════════════════════════════════════════════════════════");
        println!("  VERDICT: NOT FIXED YET");
        println!("══════════════════════════════════════════════════════════");
    }
}
