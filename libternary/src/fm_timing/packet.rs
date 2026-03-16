// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
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

use crate::TernaryTrit;

/// FM timing packet: carries both time reference AND network state
/// in a single ternary-encoded structure.
///
/// The instantaneous frequency of the trit-stream carries information —
/// this is the applied realization of Φ_T(η,θ,ψ,t).
#[derive(Clone, Debug)]
pub struct FmTimingPacket {
    /// 3³ = 27 trits encode the timestamp in balanced ternary
    pub timestamp_trits: [TernaryTrit; 27],
    /// Current instantaneous frequency and spectral state
    pub frequency_state: FrequencyState,
    /// β = A_m/ω_m quantized to u8 (modulation index)
    pub modulation_index: u8,
    /// Aggregate tonal state of the originating node
    pub network_health: TernaryTrit,
    /// Entropy nonce derived from HRV noise source
    pub entropy_nonce: [u8; 8],
}

/// Instantaneous frequency state of the FM timing oscillator.
#[derive(Clone, Debug)]
pub struct FrequencyState {
    /// Instantaneous frequency in Hz
    pub f_inst: f64,
    /// Power at sidebands f₀ ± n·f_m (n = 1..4)
    pub sidebands: [f64; 4],
    /// Synchronization quality metric: 0.0 = no sync, 1.0 = perfect lock
    pub coherence: f64,
}

/// Errors that can occur during FM timing packet encoding or decoding.
#[derive(Debug, PartialEq)]
pub enum PacketError {
    /// The encoded byte slice is shorter than the minimum packet length.
    TooShort,
    /// A trit value outside the valid balanced range was encountered.
    InvalidTrit,
    /// The coherence metric is outside the valid [0.0, 1.0] range.
    InvalidCoherence,
}

impl FmTimingPacket {
    /// Encode a timestamp (nanoseconds since HPTP epoch) into 27 balanced trits.
    ///
    /// 27 balanced trits cover ±(3^27 - 1)/2 ≈ ±3.8 trillion — sufficient for
    /// sub-epoch timing offsets in nanoseconds.
    pub fn encode_timestamp(nanos: i64) -> [TernaryTrit; 27] {
        let mut trits = [TernaryTrit::Zero; 27];
        let mut val = nanos;
        for trit in trits.iter_mut() {
            let rem = val.rem_euclid(3);
            *trit = match rem {
                0 => TernaryTrit::Zero,
                1 => TernaryTrit::Pos,
                2 => {
                    val += 1;
                    TernaryTrit::Neg
                }
                _ => unreachable!(),
            };
            val = val.div_euclid(3);
        }
        trits
    }

    /// Decode 27 balanced trits back to nanoseconds
    pub fn decode_timestamp(trits: &[TernaryTrit; 27]) -> i64 {
        let mut result: i64 = 0;
        let mut power: i64 = 1;
        for trit in trits.iter() {
            result += match trit {
                TernaryTrit::Neg => -1i64,
                TernaryTrit::Zero => 0i64,
                TernaryTrit::Pos => 1i64,
            } * power;
            power *= 3;
        }
        result
    }

    /// Serialize to bytes for network transmission.
    ///
    /// Layout:
    ///   [0..7]   27 trits packed as 2 bits each (7 bytes, 56 bits, 2 unused)
    ///   [7..15]  f_inst as f64 LE
    ///   [15..47] 4 sidebands as f64 LE (32 bytes)
    ///   [47..55] coherence as f64 LE
    ///   [55]     modulation_index u8
    ///   [56]     network_health trit as u8 (0=neg, 1=zero, 2=pos)
    ///   [57..65] entropy_nonce
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(65);

        for chunk in self.timestamp_trits.chunks(4) {
            let mut byte: u8 = 0;
            for (i, trit) in chunk.iter().enumerate() {
                let val: u8 = match trit {
                    TernaryTrit::Neg => 0b00,
                    TernaryTrit::Zero => 0b01,
                    TernaryTrit::Pos => 0b10,
                };
                byte |= val << (i * 2);
            }
            buf.push(byte);
        }

        buf.extend_from_slice(&self.frequency_state.f_inst.to_le_bytes());
        for sb in &self.frequency_state.sidebands {
            buf.extend_from_slice(&sb.to_le_bytes());
        }
        buf.extend_from_slice(&self.frequency_state.coherence.to_le_bytes());
        buf.push(self.modulation_index);
        buf.push(match self.network_health {
            TernaryTrit::Neg => 0,
            TernaryTrit::Zero => 1,
            TernaryTrit::Pos => 2,
        });
        buf.extend_from_slice(&self.entropy_nonce);
        buf
    }

    /// Deserialize from network bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, PacketError> {
        if data.len() < 65 {
            return Err(PacketError::TooShort);
        }

        let mut timestamp_trits = [TernaryTrit::Zero; 27];
        let mut trit_idx = 0;
        for &byte in &data[..7] {
            for i in 0..4 {
                if trit_idx >= 27 {
                    break;
                }
                timestamp_trits[trit_idx] = match (byte >> (i * 2)) & 0b11 {
                    0b00 => TernaryTrit::Neg,
                    0b01 => TernaryTrit::Zero,
                    0b10 => TernaryTrit::Pos,
                    _ => return Err(PacketError::InvalidTrit),
                };
                trit_idx += 1;
            }
        }

        let f_inst = f64::from_le_bytes(data[7..15].try_into().unwrap());
        let mut sidebands = [0.0f64; 4];
        for (i, sb) in sidebands.iter_mut().enumerate() {
            let start = 15 + i * 8;
            *sb = f64::from_le_bytes(data[start..start + 8].try_into().unwrap());
        }
        let coherence = f64::from_le_bytes(data[47..55].try_into().unwrap());
        if !(0.0..=1.0).contains(&coherence) {
            return Err(PacketError::InvalidCoherence);
        }

        let modulation_index = data[55];
        let network_health = match data[56] {
            0 => TernaryTrit::Neg,
            1 => TernaryTrit::Zero,
            2 => TernaryTrit::Pos,
            _ => return Err(PacketError::InvalidTrit),
        };

        let mut entropy_nonce = [0u8; 8];
        entropy_nonce.copy_from_slice(&data[57..65]);

        Ok(Self {
            timestamp_trits,
            frequency_state: FrequencyState {
                f_inst,
                sidebands,
                coherence,
            },
            modulation_index,
            network_health,
            entropy_nonce,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_roundtrip() {
        let test_values: &[i64] = &[
            0,
            1,
            -1,
            42,
            -42,
            1_000_000,
            -1_000_000,
            3_652_500_000,
            -3_652_500_000,
        ];
        for &val in test_values {
            let trits = FmTimingPacket::encode_timestamp(val);
            let decoded = FmTimingPacket::decode_timestamp(&trits);
            assert_eq!(val, decoded, "Failed roundtrip for {}", val);
        }
    }

    #[test]
    fn timestamp_range_covers_hptp_needs() {
        let max = (3i64.pow(27) - 1) / 2;
        assert!(
            max > 3_600_000_000_000i64,
            "Range {} must exceed 1hr in nanos",
            max
        );
    }

    #[test]
    fn packet_serialization_roundtrip() {
        let packet = FmTimingPacket {
            timestamp_trits: FmTimingPacket::encode_timestamp(42_000_000),
            frequency_state: FrequencyState {
                f_inst: 1.0,
                sidebands: [0.1, 0.05, 0.02, 0.01],
                coherence: 0.95,
            },
            modulation_index: 128,
            network_health: TernaryTrit::Pos,
            entropy_nonce: [1, 2, 3, 4, 5, 6, 7, 8],
        };

        let bytes = packet.to_bytes();
        let decoded = FmTimingPacket::from_bytes(&bytes).unwrap();

        let original_ts = FmTimingPacket::decode_timestamp(&packet.timestamp_trits);
        let decoded_ts = FmTimingPacket::decode_timestamp(&decoded.timestamp_trits);
        assert_eq!(original_ts, decoded_ts);

        assert!((decoded.frequency_state.f_inst - 1.0).abs() < 1e-10);
        assert!((decoded.frequency_state.coherence - 0.95).abs() < 1e-10);
        assert_eq!(decoded.modulation_index, 128);
        assert_eq!(decoded.network_health, TernaryTrit::Pos);
        assert_eq!(decoded.entropy_nonce, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn packet_rejects_too_short() {
        assert!(matches!(
            FmTimingPacket::from_bytes(&[0u8; 10]),
            Err(PacketError::TooShort)
        ));
    }

    #[test]
    fn packet_rejects_invalid_coherence() {
        let packet = FmTimingPacket {
            timestamp_trits: [TernaryTrit::Zero; 27],
            frequency_state: FrequencyState {
                f_inst: 1.0,
                sidebands: [0.0; 4],
                coherence: 0.5,
            },
            modulation_index: 0,
            network_health: TernaryTrit::Zero,
            entropy_nonce: [0u8; 8],
        };
        let mut bytes = packet.to_bytes();
        let bad_coherence = 2.0f64.to_le_bytes();
        bytes[47..55].copy_from_slice(&bad_coherence);
        assert!(matches!(
            FmTimingPacket::from_bytes(&bytes),
            Err(PacketError::InvalidCoherence)
        ));
    }

    #[test]
    fn all_trit_values_survive_serialization() {
        let mut trits = [TernaryTrit::Zero; 27];
        trits[0] = TernaryTrit::Neg;
        trits[1] = TernaryTrit::Zero;
        trits[2] = TernaryTrit::Pos;

        let packet = FmTimingPacket {
            timestamp_trits: trits,
            frequency_state: FrequencyState {
                f_inst: 0.0,
                sidebands: [0.0; 4],
                coherence: 0.0,
            },
            modulation_index: 0,
            network_health: TernaryTrit::Zero,
            entropy_nonce: [0u8; 8],
        };

        let bytes = packet.to_bytes();
        let decoded = FmTimingPacket::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.timestamp_trits[0], TernaryTrit::Neg);
        assert_eq!(decoded.timestamp_trits[1], TernaryTrit::Zero);
        assert_eq!(decoded.timestamp_trits[2], TernaryTrit::Pos);
    }
}
