use crate::{pow10, Decimal};
use cosmwasm_std::Uint128;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Custom serialization for Decimal<D> to match cosmwasm_std::Decimal's format
///
/// Serializes as a string with 18 decimal places (e.g., "1.500000000000000000")
/// even though internally we may store fewer decimal places.
/// This ensures storage compatibility with cosmwasm_std::Decimal.
impl<const D: u32> Serialize for Decimal<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Get integer and fractional parts from D-decimal atomics
        let integer = self.0.u128() / Self::FRACTIONAL;
        let fraction_d = self.0.u128() % Self::FRACTIONAL;

        // Format similar to cosmwasm_std::Decimal - compact format without trailing zeros
        if fraction_d == 0 {
            // No fractional part, just output the integer
            serializer.serialize_str(&integer.to_string())
        } else {
            // Scale the fractional part from D decimals to 18 decimals
            let fraction_18 = if D >= 18 {
                fraction_d / pow10(D - 18)
            } else {
                fraction_d * pow10(18 - D)
            };

            // Format with 18 decimal places and trim trailing zeros
            let frac_str = format!("{:0>18}", fraction_18);
            let trimmed = frac_str.trim_end_matches('0');

            serializer.serialize_str(&format!("{}.{}", integer, trimmed))
        }
    }
}

/// Custom deserialization for Decimal<D> to accept cosmwasm_std::Decimal's format
///
/// Accepts strings in the format "1.500000000000000000" (18 decimals)
/// or shorter formats like "1.5", and scales to D decimals internally.
impl<'de, const D: u32> Deserialize<'de> for Decimal<D> {
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor::<D>)
    }
}

struct DecimalVisitor<const D: u32>;

impl<'de, const D: u32> de::Visitor<'de> for DecimalVisitor<D> {
    type Value = Decimal<D>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a decimal number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Parse the string as a decimal number
        let parts: Vec<&str> = v.split('.').collect();

        match parts.len() {
            1 => {
                // Integer only, e.g., "123"
                let integer = parts[0]
                    .parse::<u128>()
                    .map_err(|_| E::custom(format!("Invalid integer part: {}", parts[0])))?;

                Ok(Decimal(Uint128::from(
                    integer * Decimal::<D>::FRACTIONAL,
                )))
            }
            2 => {
                // Integer and fractional parts, e.g., "123.456" or "1.500000000000000000"
                let integer = parts[0]
                    .parse::<u128>()
                    .map_err(|_| E::custom(format!("Invalid integer part: {}", parts[0])))?;

                let fractional_str = parts[1];

                // Handle fractional part - could be 18 decimals (from Decimal) or fewer
                let fractional_value = if fractional_str.len() <= D as usize {
                    // Short format like "1.5" or format with D or fewer decimals
                    let frac = fractional_str
                        .parse::<u128>()
                        .map_err(|_| E::custom(format!("Invalid fractional part: {}", fractional_str)))?;

                    // Scale to D decimals
                    frac * pow10(D - fractional_str.len() as u32)
                } else {
                    // Long format (more decimals than D)
                    // Parse and scale down to D decimals
                    let frac = fractional_str
                        .parse::<u128>()
                        .map_err(|_| E::custom(format!("Invalid fractional part: {}", fractional_str)))?;

                    // Scale down from input decimals to D decimals
                    let input_decimals = fractional_str.len() as u32;
                    frac / pow10(input_decimals - D)
                };

                let total_atomics = integer
                    .checked_mul(Decimal::<D>::FRACTIONAL)
                    .and_then(|i| i.checked_add(fractional_value))
                    .ok_or_else(|| E::custom("Overflow in decimal value"))?;

                Ok(Decimal(Uint128::from(total_atomics)))
            }
            _ => Err(E::custom(format!("Invalid decimal format: {}", v))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decimal6, Decimal9, Decimal18};
    use serde_json;

    #[test]
    fn test_serialize_compact_format() {
        let custom = Decimal::<6>::raw(1_500_000); // 1.5 in 6 decimals
        let json = serde_json::to_string(&custom).unwrap();
        assert_eq!(json, r#""1.5""#);
    }

    #[test]
    fn test_serialize_zero() {
        let custom = Decimal::<6>::raw(0);
        let json = serde_json::to_string(&custom).unwrap();
        assert_eq!(json, r#""0""#);
    }

    #[test]
    fn test_serialize_one() {
        let custom = Decimal::<6>::raw(1_000_000); // 1.0 in 6 decimals
        let json = serde_json::to_string(&custom).unwrap();
        assert_eq!(json, r#""1""#);
    }

    #[test]
    fn test_deserialize_18_decimal_format() {
        let json = r#""1.500000000000000000""#;
        let custom: Decimal<6> = serde_json::from_str(json).unwrap();
        assert_eq!(custom.0, Uint128::new(1_500_000)); // 1.5 in 6 decimals
    }

    #[test]
    fn test_deserialize_short_format() {
        let json = r#""1.5""#;
        let custom: Decimal<6> = serde_json::from_str(json).unwrap();
        assert_eq!(custom.0, Uint128::new(1_500_000)); // 1.5 in 6 decimals
    }

    #[test]
    fn test_deserialize_integer_only() {
        let json = r#""123""#;
        let custom: Decimal<6> = serde_json::from_str(json).unwrap();
        assert_eq!(custom.0, Uint128::new(123_000_000)); // 123.0 in 6 decimals
    }

    #[test]
    fn test_roundtrip() {
        let original = Decimal::<6>::raw(1_234_567); // 1.234567 in 6 decimals
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Decimal<6> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_deserialize_with_trailing_zeros() {
        let json = r#""1.123000000000000000""#;
        let custom: Decimal<6> = serde_json::from_str(json).unwrap();
        assert_eq!(custom.0, Uint128::new(1_123_000)); // 1.123 in 6 decimals
    }

    #[test]
    fn test_deserialize_precision_loss() {
        // When deserializing from 18 decimals, we lose precision beyond 6 decimals
        let json = r#""1.123456789012345678""#;
        let custom: Decimal<6> = serde_json::from_str(json).unwrap();
        // Should truncate to 1.123456
        assert_eq!(custom.0, Uint128::new(1_123_456));
    }

    // ========== Decimal9 tests ==========

    #[test]
    fn test_decimal9_serialize() {
        let d9 = Decimal9::raw(1_500_000_000); // 1.5 in 9 decimals
        let json = serde_json::to_string(&d9).unwrap();
        assert_eq!(json, r#""1.5""#);
    }

    #[test]
    fn test_decimal9_deserialize() {
        let json = r#""1.5""#;
        let d9: Decimal9 = serde_json::from_str(json).unwrap();
        assert_eq!(d9.0, Uint128::new(1_500_000_000));
    }

    #[test]
    fn test_decimal9_roundtrip() {
        let original = Decimal9::raw(1_234_567_890); // 1.23456789 in 9 decimals
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Decimal9 = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_decimal9_from_18_decimals() {
        let json = r#""1.123456789012345678""#;
        let d9: Decimal9 = serde_json::from_str(json).unwrap();
        // Should truncate to 9 decimals: 1.123456789
        assert_eq!(d9.0, Uint128::new(1_123_456_789));
    }

    // ========== Decimal18 tests ==========

    #[test]
    fn test_decimal18_serialize() {
        let d18 = Decimal18::raw(1_500_000_000_000_000_000); // 1.5 in 18 decimals
        let json = serde_json::to_string(&d18).unwrap();
        assert_eq!(json, r#""1.5""#);
    }

    #[test]
    fn test_decimal18_deserialize() {
        let json = r#""1.5""#;
        let d18: Decimal18 = serde_json::from_str(json).unwrap();
        assert_eq!(d18.0, Uint128::new(1_500_000_000_000_000_000));
    }

    #[test]
    fn test_decimal18_full_precision() {
        let json = r#""1.123456789012345678""#;
        let d18: Decimal18 = serde_json::from_str(json).unwrap();
        assert_eq!(d18.0, Uint128::new(1_123_456_789_012_345_678));
    }

    // ========== Cross-precision compatibility tests ==========

    #[test]
    fn test_decimal6_decimal18_compatibility() {
        // Serialize Decimal6
        let d6 = Decimal6::raw(1_500_000); // 1.5
        let json = serde_json::to_string(&d6).unwrap();

        // Deserialize as Decimal18
        let d18: Decimal18 = serde_json::from_str(&json).unwrap();
        assert_eq!(d18.0, Uint128::new(1_500_000_000_000_000_000)); // 1.5 in 18 decimals
    }

    #[test]
    fn test_decimal18_decimal6_compatibility() {
        // Serialize Decimal18
        let d18 = Decimal18::raw(1_500_000_000_000_000_000); // 1.5
        let json = serde_json::to_string(&d18).unwrap();

        // Deserialize as Decimal6 (with precision loss)
        let d6: Decimal6 = serde_json::from_str(&json).unwrap();
        assert_eq!(d6.0, Uint128::new(1_500_000)); // 1.5 in 6 decimals
    }
}
