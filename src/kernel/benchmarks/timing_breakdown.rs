use std::hint::black_box;
use plenumnet_kernel::crypto::tl_dsa::{self, TlDsaVariant};

fn main() {
    let seed = vec![-1i8, 0, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1];
    let msg = vec![1i8, 0, -1, 1, 0, -1, 1, 0];

    for &(variant, label) in &[
        (TlDsaVariant::TlDsa44, "TL-DSA-44"),
        (TlDsaVariant::TlDsa65, "TL-DSA-65"),
        (TlDsaVariant::TlDsa87, "TL-DSA-87"),
    ] {
        println!("\n{} sign+verify breakdown:", label);
        println!("{}", "─".repeat(60));
        let timings = tl_dsa::sign_verify_timing_breakdown(variant, &seed, &msg).unwrap();
        let total: std::time::Duration = timings.iter().map(|(_, d)| *d).sum();
        let total_us = total.as_micros() as f64;
        for (name, dur) in &timings {
            let us = dur.as_micros() as f64;
            let pct = if total_us > 0.0 { us / total_us * 100.0 } else { 0.0 };
            let bar_len = (pct / 2.0) as usize;
            let bar: String = "█".repeat(bar_len);
            println!("  {:.<30} {:>10.0} µs  ({:>5.1}%) {}", name, us, pct, bar);
        }
        println!("  {:.<30} {:>10.0} µs  (100.0%)", "TOTAL", total_us);
        black_box(&timings);
    }
}
