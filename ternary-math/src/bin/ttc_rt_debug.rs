use ternary_math::ttc::{ttc_compress, ttc_decompress, CompressOptions};

fn test_random(size: usize, trial: usize) -> (bool, usize, usize) {
    let mut data = vec![0u8; size];
    getrandom::getrandom(&mut data).expect("getrandom failed");

    let opts = CompressOptions {
        filename: Some(format!("random_{}_{}", size, trial)),
        ..Default::default()
    };
    let result = ttc_compress(&data, &opts).expect("compress failed");
    let dec = ttc_decompress(&result.compressed).expect("decompress failed");

    let pass = dec.data == data;
    if !pass {
        let min_len = std::cmp::min(data.len(), dec.data.len());
        let mut diffs = 0;
        let mut first_diff = None;
        for i in 0..min_len {
            if data[i] != dec.data[i] {
                if first_diff.is_none() { first_diff = Some(i); }
                diffs += 1;
            }
        }
        let len_diff = (data.len() as i64 - dec.data.len() as i64).unsigned_abs() as usize;
        eprintln!("  FAIL size={} trial={}: orig={} dec={} compressed={} diffs={} len_delta={} first_diff={:?}",
            size, trial, data.len(), dec.data.len(), result.compressed_size,
            diffs + len_diff, len_diff, first_diff);
    }
    (pass, data.len(), result.compressed_size as usize)
}

fn main() {
    println!("TTC Round-Trip Integrity Test — OS Random Data");
    println!("================================================\n");

    for &size in &[512, 1024, 2048, 4096, 8192, 10240, 16384, 32768, 50000, 65536, 100000] {
        let mut pass_count = 0;
        let trials = 5;
        for trial in 0..trials {
            let (pass, _, _) = test_random(size, trial);
            if pass { pass_count += 1; }
        }
        let status = if pass_count == trials { "✓ ALL PASS" } else { "✗ FAILURES" };
        println!("  size={:>7}: {}/{} passed  {}", size, pass_count, trials, status);
    }
}
