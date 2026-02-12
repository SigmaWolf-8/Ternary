// Copyright (c) 2025–2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

#![no_main]
use libfuzzer_sys::fuzz_target;
use plenumnet_kernel::compat::gateway::{
    binary_to_balanced_ternary, balanced_ternary_to_binary,
    binary_bytes_to_ternary, ternary_to_binary_bytes,
    binary_u8_to_representation_b, representation_b_to_binary_u8,
    BinaryTernaryGateway, GatewayMode,
};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    if data.len() >= 8 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[..8]);
        let value = i64::from_le_bytes(bytes);

        let trits = binary_to_balanced_ternary(value);

        for &trit in &trits {
            assert!(trit >= -1 && trit <= 1, "balanced ternary produced invalid trit: {}", trit);
        }

        if !trits.is_empty() {
            let back = balanced_ternary_to_binary(&trits);
            match back {
                Ok(recovered) => {
                    assert_eq!(recovered, value, "i64 roundtrip failed: {} != {}", recovered, value);
                }
                Err(_) => {
                    panic!("balanced_ternary_to_binary failed for valid trits from binary_to_balanced_ternary({})", value);
                }
            }
        } else {
            assert_eq!(value, 0, "Empty trit vector produced for non-zero value: {}", value);
        }
    }

    {
        let trits = binary_bytes_to_ternary(data);
        assert_eq!(trits.len(), data.len() * 6, "bytes->trits: wrong trit count");

        for &trit in &trits {
            assert!(trit >= -1 && trit <= 1, "binary_bytes_to_ternary produced invalid trit: {}", trit);
        }

        let recovered = ternary_to_binary_bytes(&trits);
        match recovered {
            Ok(bytes) => {
                assert_eq!(bytes, data, "bytes roundtrip failed");
            }
            Err(_) => {
                panic!("ternary_to_binary_bytes failed for valid trits from binary_bytes_to_ternary");
            }
        }
    }

    for &byte in data.iter().take(32) {
        let rep_b = binary_u8_to_representation_b(byte);
        for &digit in &rep_b {
            assert!(digit <= 2, "rep_b produced invalid digit: {}", digit);
        }
        let back = representation_b_to_binary_u8(&rep_b).unwrap();
        assert_eq!(back, byte, "rep_b roundtrip failed for byte {}", byte);
    }

    if data.len() >= 6 {
        let trit_stream: Vec<i8> = data.iter()
            .take(data.len() - (data.len() % 6))
            .map(|&b| (b % 3) as i8 - 1)
            .collect();

        if trit_stream.len() % 6 == 0 && !trit_stream.is_empty() {
            let result = ternary_to_binary_bytes(&trit_stream);
            match result {
                Ok(bytes) => {
                    assert!(bytes.len() == trit_stream.len() / 6);
                    for &b in &bytes {
                        assert!(b <= 255);
                    }
                }
                Err(_) => {
                }
            }
        }
    }

    if data.len() >= 2 {
        let mut gw = BinaryTernaryGateway::new(match data[0] % 3 {
            0 => GatewayMode::Strict,
            1 => GatewayMode::Lossy,
            _ => GatewayMode::Balanced,
        });

        let chunk = &data[1..];
        if let Ok(trits) = gw.convert_to_ternary(chunk) {
            if let Ok(back) = gw.convert_to_binary(&trits) {
                assert_eq!(back.as_slice(), chunk, "Gateway roundtrip failed");
            }
        }

        let stats = gw.stats();
        assert!(stats.conversions >= 1);
    }

    {
        let random_trits: Vec<i8> = data.iter().map(|&b| {
            match b % 5 {
                0 => -2,
                1 => -1,
                2 => 0,
                3 => 1,
                _ => 2,
            }
        }).collect();

        let result = balanced_ternary_to_binary(&random_trits);
        let has_invalid = random_trits.iter().any(|&t| t < -1 || t > 1);
        if has_invalid {
            assert!(result.is_err(), "Should reject trits with values outside [-1, 1]");
        }
    }
});
