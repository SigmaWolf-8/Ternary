use alloc::string::String;
use alloc::vec::Vec;
use super::{CompatError, CompatResult};

pub fn binary_to_balanced_ternary(value: i64) -> Vec<i8> {
    if value == 0 {
        return Vec::new();
    }

    let negative = value < 0;
    let mut n = if value == i64::MIN {
        (i64::MAX as u64) + 1
    } else {
        value.unsigned_abs()
    };

    let mut trits = Vec::new();
    while n > 0 {
        let remainder = (n % 3) as i8;
        if remainder == 2 {
            trits.push(-1);
            n = n / 3 + 1;
        } else {
            trits.push(remainder);
            n /= 3;
        }
    }

    if negative {
        for trit in trits.iter_mut() {
            *trit = -*trit;
        }
    }

    trits
}

pub fn balanced_ternary_to_binary(trits: &[i8]) -> CompatResult<i64> {
    let mut result: i64 = 0;
    let mut power: i64 = 1;

    for (i, &trit) in trits.iter().enumerate() {
        if trit < -1 || trit > 1 {
            return Err(CompatError::InvalidTernaryData);
        }
        result = result.checked_add(power.checked_mul(trit as i64).ok_or(CompatError::OverflowError)?)
            .ok_or(CompatError::OverflowError)?;
        if i + 1 < trits.len() {
            power = power.checked_mul(3).ok_or(CompatError::OverflowError)?;
        }
    }

    Ok(result)
}

pub fn binary_bytes_to_ternary(data: &[u8]) -> Vec<i8> {
    let mut trits = Vec::with_capacity(data.len() * 6);
    for &byte in data {
        let mut val = byte as u16;
        for _ in 0..6 {
            let remainder = (val % 3) as i8;
            let trit = if remainder == 2 {
                val = val / 3 + 1;
                -1
            } else {
                val /= 3;
                remainder
            };
            trits.push(trit);
        }
    }
    trits
}

pub fn ternary_to_binary_bytes(trits: &[i8]) -> CompatResult<Vec<u8>> {
    if trits.len() % 6 != 0 {
        return Err(CompatError::InvalidTernaryData);
    }

    let mut bytes = Vec::with_capacity(trits.len() / 6);
    for chunk in trits.chunks(6) {
        let mut value: i64 = 0;
        let mut power: i64 = 1;
        for &trit in chunk {
            if trit < -1 || trit > 1 {
                return Err(CompatError::InvalidTernaryData);
            }
            value += trit as i64 * power;
            power *= 3;
        }
        if value < 0 || value > 255 {
            return Err(CompatError::InvalidBinaryData);
        }
        bytes.push(value as u8);
    }
    Ok(bytes)
}

pub fn binary_u8_to_representation_b(byte: u8) -> [u8; 6] {
    let mut digits = [0u8; 6];
    let mut val = byte as u16;
    for i in 0..6 {
        digits[i] = (val % 3) as u8;
        val /= 3;
    }
    digits
}

pub fn representation_b_to_binary_u8(digits: &[u8; 6]) -> CompatResult<u8> {
    let mut value: u16 = 0;
    let mut power: u16 = 1;
    for &digit in digits {
        if digit > 2 {
            return Err(CompatError::InvalidTernaryData);
        }
        value += digit as u16 * power;
        power *= 3;
    }
    if value > 255 {
        return Err(CompatError::InvalidBinaryData);
    }
    Ok(value as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayMode {
    Strict,
    Lossy,
    Balanced,
}

#[derive(Debug, Clone)]
pub struct GatewayStats {
    pub conversions: u64,
    pub bytes_processed: u64,
    pub precision_losses: u64,
}

pub struct BinaryTernaryGateway {
    conversions_count: u64,
    bytes_converted: u64,
    precision_losses: u64,
    mode: GatewayMode,
}

impl BinaryTernaryGateway {
    pub fn new(mode: GatewayMode) -> Self {
        Self {
            conversions_count: 0,
            bytes_converted: 0,
            precision_losses: 0,
            mode,
        }
    }

    pub fn convert_to_ternary(&mut self, data: &[u8]) -> CompatResult<Vec<i8>> {
        self.conversions_count += 1;
        self.bytes_converted += data.len() as u64;
        Ok(binary_bytes_to_ternary(data))
    }

    pub fn convert_to_binary(&mut self, trits: &[i8]) -> CompatResult<Vec<u8>> {
        let result = ternary_to_binary_bytes(trits)?;
        self.conversions_count += 1;
        self.bytes_converted += result.len() as u64;
        Ok(result)
    }

    pub fn convert_i64_to_ternary(&mut self, value: i64) -> Vec<i8> {
        self.conversions_count += 1;
        binary_to_balanced_ternary(value)
    }

    pub fn convert_ternary_to_i64(&mut self, trits: &[i8]) -> CompatResult<i64> {
        self.conversions_count += 1;
        balanced_ternary_to_binary(trits)
    }

    pub fn convert_f64_to_ternary(&mut self, value: f64, precision_trits: usize) -> Vec<i8> {
        self.conversions_count += 1;
        let integer_part = value as i64;
        let mut trits = binary_to_balanced_ternary(integer_part);

        let mut frac = (value - integer_part as f64).abs();
        if frac > 0.0 {
            self.precision_losses += 1;
        }

        let int_len = trits.len();
        for _ in 0..precision_trits {
            frac *= 3.0;
            let digit = frac as i64;
            let trit = if digit >= 2 {
                frac -= 2.0;
                -1i8
            } else if digit == 1 {
                frac -= 1.0;
                1i8
            } else {
                0i8
            };
            trits.push(trit);
        }

        let _ = int_len;
        trits
    }

    pub fn convert_ternary_to_f64(&self, trits: &[i8], decimal_point: usize) -> f64 {
        let int_trits = if decimal_point <= trits.len() {
            &trits[..decimal_point]
        } else {
            trits
        };

        let mut int_val: f64 = 0.0;
        let mut power: f64 = 1.0;
        for &trit in int_trits {
            int_val += trit as f64 * power;
            power *= 3.0;
        }

        let mut frac_val: f64 = 0.0;
        if decimal_point < trits.len() {
            let mut frac_power: f64 = 1.0 / 3.0;
            for &trit in &trits[decimal_point..] {
                frac_val += trit as f64 * frac_power;
                frac_power /= 3.0;
            }
        }

        int_val + frac_val
    }

    pub fn stats(&self) -> GatewayStats {
        GatewayStats {
            conversions: self.conversions_count,
            bytes_processed: self.bytes_converted,
            precision_losses: self.precision_losses,
        }
    }

    pub fn reset_stats(&mut self) {
        self.conversions_count = 0;
        self.bytes_converted = 0;
        self.precision_losses = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_to_balanced_ternary_zero() {
        let trits = binary_to_balanced_ternary(0);
        assert!(trits.is_empty());
    }

    #[test]
    fn test_binary_to_balanced_ternary_one() {
        let trits = binary_to_balanced_ternary(1);
        assert_eq!(trits, [1]);
    }

    #[test]
    fn test_binary_to_balanced_ternary_neg_one() {
        let trits = binary_to_balanced_ternary(-1);
        assert_eq!(trits, [-1]);
    }

    #[test]
    fn test_binary_to_balanced_ternary_five() {
        let trits = binary_to_balanced_ternary(5);
        assert_eq!(trits, [-1, -1, 1]);
        let back = balanced_ternary_to_binary(&trits).unwrap();
        assert_eq!(back, 5);
    }

    #[test]
    fn test_binary_to_balanced_ternary_thirteen() {
        let trits = binary_to_balanced_ternary(13);
        let back = balanced_ternary_to_binary(&trits).unwrap();
        assert_eq!(back, 13);
    }

    #[test]
    fn test_binary_to_balanced_ternary_two() {
        let trits = binary_to_balanced_ternary(2);
        assert_eq!(trits, [-1, 1]);
        let back = balanced_ternary_to_binary(&trits).unwrap();
        assert_eq!(back, 2);
    }

    #[test]
    fn test_binary_to_balanced_ternary_three() {
        let trits = binary_to_balanced_ternary(3);
        assert_eq!(trits, [0, 1]);
    }

    #[test]
    fn test_balanced_ternary_roundtrip_positive() {
        for v in [1, 2, 3, 4, 5, 10, 13, 42, 100, 255, 1000, 9999] {
            let trits = binary_to_balanced_ternary(v);
            let back = balanced_ternary_to_binary(&trits).unwrap();
            assert_eq!(back, v, "Roundtrip failed for {}", v);
        }
    }

    #[test]
    fn test_balanced_ternary_roundtrip_negative() {
        for v in [-1, -2, -5, -13, -42, -100, -1000] {
            let trits = binary_to_balanced_ternary(v);
            let back = balanced_ternary_to_binary(&trits).unwrap();
            assert_eq!(back, v, "Roundtrip failed for {}", v);
        }
    }

    #[test]
    fn test_balanced_ternary_to_binary_invalid_trit() {
        let trits = [0, 2, 1];
        assert!(balanced_ternary_to_binary(&trits).is_err());
    }

    #[test]
    fn test_balanced_ternary_to_binary_empty() {
        let trits: &[i8] = &[];
        let result = balanced_ternary_to_binary(trits).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_binary_bytes_to_ternary_zero() {
        let trits = binary_bytes_to_ternary(&[0x00]);
        assert_eq!(trits.len(), 6);
        let bytes = ternary_to_binary_bytes(&trits).unwrap();
        assert_eq!(bytes, [0x00]);
    }

    #[test]
    fn test_binary_bytes_to_ternary_ff() {
        let trits = binary_bytes_to_ternary(&[0xFF]);
        assert_eq!(trits.len(), 6);
        let bytes = ternary_to_binary_bytes(&trits).unwrap();
        assert_eq!(bytes, [0xFF]);
    }

    #[test]
    fn test_binary_bytes_roundtrip() {
        let original = [0u8, 1, 127, 128, 255, 42, 73];
        let trits = binary_bytes_to_ternary(&original);
        let recovered = ternary_to_binary_bytes(&trits).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_ternary_to_binary_bytes_invalid_length() {
        let trits = [0i8, 1, -1, 0, 1];
        assert!(ternary_to_binary_bytes(&trits).is_err());
    }

    #[test]
    fn test_representation_b_zero() {
        let digits = binary_u8_to_representation_b(0);
        assert_eq!(digits, [0, 0, 0, 0, 0, 0]);
        let back = representation_b_to_binary_u8(&digits).unwrap();
        assert_eq!(back, 0);
    }

    #[test]
    fn test_representation_b_roundtrip() {
        for byte in [0u8, 1, 2, 42, 127, 128, 200, 255] {
            let digits = binary_u8_to_representation_b(byte);
            let back = representation_b_to_binary_u8(&digits).unwrap();
            assert_eq!(back, byte, "Rep B roundtrip failed for {}", byte);
        }
    }

    #[test]
    fn test_representation_b_invalid_digit() {
        let digits = [0, 1, 3, 0, 0, 0];
        assert!(representation_b_to_binary_u8(&digits).is_err());
    }

    #[test]
    fn test_gateway_convert_to_ternary_roundtrip() {
        let mut gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
        let data = [42u8, 0, 255];
        let trits = gw.convert_to_ternary(&data).unwrap();
        let back = gw.convert_to_binary(&trits).unwrap();
        assert_eq!(back, data);
    }

    #[test]
    fn test_gateway_stats_tracking() {
        let mut gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
        let data = [1u8, 2, 3];
        let _ = gw.convert_to_ternary(&data).unwrap();
        let stats = gw.stats();
        assert_eq!(stats.conversions, 1);
        assert_eq!(stats.bytes_processed, 3);
    }

    #[test]
    fn test_gateway_stats_reset() {
        let mut gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
        let _ = gw.convert_to_ternary(&[1u8]).unwrap();
        gw.reset_stats();
        let stats = gw.stats();
        assert_eq!(stats.conversions, 0);
        assert_eq!(stats.bytes_processed, 0);
    }

    #[test]
    fn test_gateway_i64_roundtrip() {
        let mut gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
        for v in [0i64, 1, -1, 42, -42, 1000, -9999] {
            let trits = gw.convert_i64_to_ternary(v);
            let back = gw.convert_ternary_to_i64(&trits).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn test_gateway_f64_conversion() {
        let mut gw = BinaryTernaryGateway::new(GatewayMode::Lossy);
        let trits = gw.convert_f64_to_ternary(3.5, 6);
        assert!(!trits.is_empty());
    }

    #[test]
    fn test_gateway_ternary_to_f64() {
        let gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
        let trits = [0i8, 1];
        let val = gw.convert_ternary_to_f64(&trits, 2);
        assert!((val - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_gateway_mode_strict() {
        let gw = BinaryTernaryGateway::new(GatewayMode::Strict);
        assert_eq!(gw.stats().conversions, 0);
    }

    #[test]
    fn test_gateway_empty_data() {
        let mut gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
        let trits = gw.convert_to_ternary(&[]).unwrap();
        assert!(trits.is_empty());
        let bytes = gw.convert_to_binary(&[]).unwrap();
        assert!(bytes.is_empty());
    }
}
