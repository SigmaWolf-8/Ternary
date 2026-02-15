// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.
//
// ============================================================================
// HPTP Jitter Correction with Qutrit Stabilizer Logic
// ============================================================================
//
// Uses the [[3,1,2]]_3 qutrit stabilizer correction principle to detect and
// correct phase-like jitter anomalies in femtosecond timestamp streams.
//
// The core idea: treat 3 consecutive timestamps as a "qutrit-encoded" triple.
// Compute syndrome-like differences between copies. If the syndrome exceeds a
// threshold, replace the outlier with the median (majority vote correction).
//
// This is real-time operational code — not a simulation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JitterCorrectionResult {
    pub corrections_applied: u64,
    pub timestamps_processed: u64,
    pub max_deviation_fs: i64,
}

pub fn correct_hptp_jitter_with_qudit(
    timestamps: &mut [i64],
    threshold: f64,
) -> JitterCorrectionResult {
    let mut result = JitterCorrectionResult {
        corrections_applied: 0,
        timestamps_processed: timestamps.len() as u64,
        max_deviation_fs: 0,
    };

    if timestamps.len() < 3 {
        return result;
    }

    let chunk_count = timestamps.len() / 3;
    for chunk_idx in 0..chunk_count {
        let base = chunk_idx * 3;
        let t0 = timestamps[base];
        let t1 = timestamps[base + 1];
        let t2 = timestamps[base + 2];

        let norm_sq = (t0 as f64) * (t0 as f64)
            + (t1 as f64) * (t1 as f64)
            + (t2 as f64) * (t2 as f64);
        let norm = libm::sqrt(norm_sq);
        if norm < 1e-15 {
            continue;
        }

        let n0 = t0 as f64 / norm;
        let n1 = t1 as f64 / norm;
        let n2 = t2 as f64 / norm;

        let s1 = libm::fabs(n0 - n1);
        let s2 = libm::fabs(n1 - n2);

        if s1 > threshold || s2 > threshold {
            let mut sorted = [t0, t1, t2];
            sorted.sort();
            let median = sorted[1];

            let dev0 = (t0 - median).unsigned_abs();
            let dev1 = (t1 - median).unsigned_abs();
            let dev2 = (t2 - median).unsigned_abs();

            let max_dev_idx = if dev0 >= dev1 && dev0 >= dev2 {
                0
            } else if dev1 >= dev0 && dev1 >= dev2 {
                1
            } else {
                2
            };

            let deviation = match max_dev_idx {
                0 => dev0,
                1 => dev1,
                _ => dev2,
            } as i64;

            if deviation > result.max_deviation_fs {
                result.max_deviation_fs = deviation;
            }

            timestamps[base + max_dev_idx] = median;
            result.corrections_applied += 1;
        }
    }

    result
}

pub fn correct_hptp_jitter_with_qudit_windowed(
    timestamps: &mut [i64],
    threshold: f64,
    window_size: usize,
) -> JitterCorrectionResult {
    let mut total_result = JitterCorrectionResult {
        corrections_applied: 0,
        timestamps_processed: timestamps.len() as u64,
        max_deviation_fs: 0,
    };

    if timestamps.len() < 3 || window_size < 3 {
        return total_result;
    }

    let effective_window = if window_size > timestamps.len() {
        timestamps.len()
    } else {
        window_size
    };

    let step = effective_window - (effective_window % 3);
    if step == 0 {
        return total_result;
    }

    let mut offset = 0;
    while offset + step <= timestamps.len() {
        let window = &mut timestamps[offset..offset + step];
        let r = correct_hptp_jitter_with_qudit(window, threshold);
        total_result.corrections_applied += r.corrections_applied;
        if r.max_deviation_fs > total_result.max_deviation_fs {
            total_result.max_deviation_fs = r.max_deviation_fs;
        }
        offset += step;
    }

    if offset < timestamps.len() && timestamps.len() - offset >= 3 {
        let remaining = &mut timestamps[offset..];
        let tail_len = remaining.len() - (remaining.len() % 3);
        if tail_len >= 3 {
            let r = correct_hptp_jitter_with_qudit(&mut remaining[..tail_len], threshold);
            total_result.corrections_applied += r.corrections_applied;
            if r.max_deviation_fs > total_result.max_deviation_fs {
                total_result.max_deviation_fs = r.max_deviation_fs;
            }
        }
    }

    total_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_jitter_no_correction() {
        let mut timestamps = [1000i64, 1000, 1000, 2000, 2000, 2000];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert_eq!(result.corrections_applied, 0);
        assert_eq!(timestamps[0], 1000);
        assert_eq!(timestamps[1], 1000);
        assert_eq!(timestamps[2], 1000);
    }

    #[test]
    fn test_single_outlier_correction() {
        let mut timestamps = [1000i64, 1000, 5000];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert!(result.corrections_applied > 0);
        assert_eq!(timestamps[2], 1000);
    }

    #[test]
    fn test_too_few_timestamps() {
        let mut timestamps = [100i64, 200];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert_eq!(result.corrections_applied, 0);
    }

    #[test]
    fn test_all_zeros() {
        let mut timestamps = [0i64, 0, 0];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert_eq!(result.corrections_applied, 0);
    }

    #[test]
    fn test_multiple_chunks() {
        let mut timestamps = [
            1000i64, 1000, 9000,
            2000, 2000, 2000,
            3000, 8000, 3000,
        ];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert!(result.corrections_applied >= 2);
        assert_eq!(timestamps[2], 1000);
        assert_eq!(timestamps[7], 3000);
    }

    #[test]
    fn test_windowed_correction() {
        let mut timestamps = [1000i64, 1000, 5000, 2000, 2000, 2000];
        let result = correct_hptp_jitter_with_qudit_windowed(&mut timestamps, 0.1, 6);
        assert!(result.corrections_applied > 0);
        assert_eq!(result.timestamps_processed, 6);
    }

    #[test]
    fn test_max_deviation_tracking() {
        let mut timestamps = [1000i64, 1000, 10000];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert!(result.max_deviation_fs > 0);
    }

    #[test]
    fn test_result_struct() {
        let r = JitterCorrectionResult {
            corrections_applied: 5,
            timestamps_processed: 30,
            max_deviation_fs: 4000,
        };
        assert_eq!(r.corrections_applied, 5);
        assert_eq!(r.timestamps_processed, 30);
        assert_eq!(r.max_deviation_fs, 4000);
    }

    #[test]
    fn test_negative_timestamps() {
        let mut timestamps = [-1000i64, -1000, -5000];
        let result = correct_hptp_jitter_with_qudit(&mut timestamps, 0.1);
        assert!(result.corrections_applied > 0);
    }
}
