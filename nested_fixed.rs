use std::time::{SystemTime, UNIX_EPOCH};

const CANDIDATES: [usize; 17] = [
    8, 13, 20, 25, 26, 27, 28, 30, 35, 36, 40, 42, 45, 46, 50, 52, 56,
];

fn detect_period(data: &[u8]) -> usize {
    let mut best_s = 1;
    let mut best_match = 0.0;
    for &s in &CANDIDATES {
        if s * s > data.len() { continue; }
        let matches = diagonal_match_rate(&data[0..s*s], s);
        if matches > best_match {
            best_match = matches;
            best_s = s;
        }
    }
    eprintln!("  detected S={} (match {:.2}%)", best_s, best_match*100.0);
    best_s
}

fn diagonal_match_rate(block: &[u8], s: usize) -> f64 {
    let mut total = 0;
    let mut matches = 0;
    for r in 0..s {
        for c in 0..r {
            if block[r*s + c] == block[c*s + r] { matches += 1; }
            total += 1;
        }
    }
    if total == 0 { 0.0 } else { matches as f64 / total as f64 }
}

#[derive(Debug)]
struct Band {
    d: usize,
    corrections: Vec<Option<u8>>,
}

impl Band {
    fn match_rate(&self) -> f64 {
        if self.corrections.is_empty() { return 0.0; }
        let matches = self.corrections.iter().filter(|c| c.is_none()).count();
        matches as f64 / self.corrections.len() as f64
    }
}

fn diagonal_fold(block: &[u8], s: usize) -> (Vec<u8>, Vec<Band>) {
    let mut upper = Vec::with_capacity(s * (s + 1) / 2);
    for r in 0..s {
        for c in r..s { upper.push(block[r*s + c]); }
    }
    let mut bands = Vec::with_capacity(s - 1);
    for d in 1..s {
        let mut corr = Vec::new();
        for r in 0..s {
            let c = r + d;
            if c < s {
                let a = block[r*s + c];
                let b = block[c*s + r];
                corr.push(if a == b { None } else { Some(a.wrapping_sub(b)) });
            }
        }
        bands.push(Band { d, corrections: corr });
    }
    (upper, bands)
}

fn encode_band_bijective(band: &Band) -> Vec<u8> {
    let mut out = Vec::new();
    let mut gap = 0;
    for corr in &band.corrections {
        match corr {
            None => gap += 1,
            Some(val) => {
                if gap > 0 {
                    out.extend(bijective_base255(gap));
                    gap = 0;
                }
                out.push(*val);
            }
        }
    }
    out
}

fn encode_band_rle(band: &Band) -> Vec<u8> {
    let mut out = Vec::new();
    let mut gap = 0;
    for corr in &band.corrections {
        match corr {
            None => gap += 1,
            Some(val) => {
                if gap > 0 {
                    out.extend(bijective_base255(gap));
                    gap = 0;
                }
                out.push(*val);
            }
        }
    }
    if gap > 0 {
        out.extend(bijective_base255(gap));
    }
    out
}

fn bijective_base255(mut n: usize) -> Vec<u8> {
    let mut digits = Vec::new();
    while n > 0 {
        let rem = n % 255;
        if rem == 0 {
            digits.push(255);
            n = n / 255 - 1;
        } else {
            digits.push(rem as u8);
            n /= 255;
        }
    }
    digits.reverse();
    digits
}

fn process_block(block: &[u8], s: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (upper, bands) = diagonal_fold(block, s);
    let mut encoded = Vec::new();
    let mut raw = Vec::new();

    for band in bands.into_iter() {
        let mr = band.match_rate();
        let is_circle = band.d % 13 == 0 || band.d % 27 == 0 || band.d % 28 == 0;

        if is_circle || mr >= 0.85 {
            encoded.extend(encode_band_bijective(&band));
        } else if mr >= 0.65 {
            encoded.extend(encode_band_rle(&band));
        } else {
            for corr in band.corrections {
                match corr {
                    None => raw.push(0),
                    Some(val) => raw.push(val),
                }
            }
        }
    }
    (upper, encoded, raw)
}

fn compress_phase(data: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut current = data.to_vec();
    let mut cumulative_encoded = Vec::new();
    let mut raw_output = Vec::new();
    let mut fold = 1;

    while fold <= 5 && current.len() >= 16 {
        let s = detect_period(&current);
        let block_size = s * s;
        if block_size > current.len() {
            break;
        }
        let num_blocks = current.len() / block_size;
        let remainder_len = current.len() % block_size;

        let mut new_kernel = Vec::new();
        let mut fold_encoded = Vec::new();
        let mut fold_raw = Vec::new();

        for i in 0..num_blocks {
            let start = i * block_size;
            let block = &current[start..start+block_size];
            let (kernel, enc, raw) = process_block(block, s);
            new_kernel.extend(kernel);
            fold_encoded.extend(enc);
            fold_raw.extend(raw);
        }

        raw_output.extend(fold_raw);

        if remainder_len > 0 {
            let start = num_blocks * block_size;
            let remainder = &current[start..];
            raw_output.extend_from_slice(remainder);
            eprintln!("    Fold {}: storing {} remainder bytes", fold, remainder_len);
        }

        cumulative_encoded.extend(fold_encoded);
        current = new_kernel;
        fold += 1;
    }

    (current, cumulative_encoded, raw_output)
}

fn current_timestamp_bytes() -> [u8; 8] {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_secs().to_be_bytes()
}

fn generate_test_data(len: usize) -> Vec<u8> {
    let record = b"{\"id\":123,\"name\":\"John\",\"value\":45}";
    (0..len).map(|i| {
        let noise = if i % 100 == 0 { i as u8 } else { 0 };
        record[i % record.len()].wrapping_add(noise % 10)
    }).collect()
}

fn main() {
    let original = generate_test_data(4800);
    let orig_len = original.len();
    println!("Original size: {} bytes", orig_len);

    let mut phases = Vec::new();
    let mut input = original;
    let mut phase_num = 1;

    while !input.is_empty() {
        println!("\n--- Phase {} (input size {} B) ---", phase_num, input.len());

        let ts_bytes = current_timestamp_bytes();
        let timestamp = u64::from_be_bytes(ts_bytes);
        println!("  Timestamp: {} (seconds since epoch)", timestamp);

        let (kernel, encoded, next_input) = compress_phase(&input);
        phases.push((ts_bytes.to_vec(), kernel.clone(), encoded.clone()));
        println!("Phase {}: kernel {} B, encoded corrections {} B, raw to next phase {} B",
                 phase_num, kernel.len(), encoded.len(), next_input.len());

        input = next_input;
        phase_num += 1;

        if phase_num > 20 {
            println!("\nSafety limit: stopping after 20 phases ({} raw bytes remain)", input.len());
            break;
        }
    }

    let total_compressed: usize = phases.iter()
        .map(|(ts, k, e)| ts.len() + k.len() + e.len())
        .sum();
    let final_ratio = orig_len as f64 / total_compressed as f64;

    println!("\n=== Final Compression ===");
    for (i, (ts, k, e)) in phases.iter().enumerate() {
        let ts_val = u64::from_be_bytes(ts.as_slice().try_into().unwrap());
        println!("Phase {}: timestamp {} B (value {}), kernel {} B, encoded {} B",
                 i+1, ts.len(), ts_val, k.len(), e.len());
    }
    println!("Total compressed size (with timestamps): {} B", total_compressed);
    println!("Compression ratio: {:.2}:1", final_ratio);
}
