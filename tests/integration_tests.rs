//! Integration tests for Decimal<D>
//!
//! These tests focus on:
//! - Storage interoperability with cosmwasm_std::Decimal
//! - Edge cases and boundary conditions
//! - Precision and rounding behavior
//! - Roundtrip conversions
//! - Multi-precision support

use cosmwasm_custom_decimal::{CustomDecimal, Decimal, Decimal6, Decimal9, Decimal12, Decimal18};
use cosmwasm_std::{Decimal as StdDecimal, Uint128};
use serde_json;
use std::str::FromStr;

// ========== Storage Interoperability Tests ==========

#[test]
fn test_serialization_matches_decimal() {
    let custom = CustomDecimal::from_str("1.5").unwrap();
    let decimal = StdDecimal::from_str("1.5").unwrap();

    let custom_json = serde_json::to_string(&custom).unwrap();
    let decimal_json = serde_json::to_string(&decimal).unwrap();

    assert_eq!(
        custom_json, decimal_json,
        "CustomDecimal and StdDecimal should serialize identically"
    );
}

#[test]
fn test_deserialize_decimal_as_custom() {
    // Serialize a StdDecimal
    let decimal = StdDecimal::from_str("1.5").unwrap();
    let json = serde_json::to_string(&decimal).unwrap();

    // Deserialize as CustomDecimal
    let custom: CustomDecimal = serde_json::from_str(&json).unwrap();

    // Should be equal (accounting for precision differences)
    assert_eq!(custom.to_string(), "1.5");
}

#[test]
fn test_deserialize_custom_as_decimal() {
    // Serialize a CustomDecimal
    let custom = CustomDecimal::from_str("1.5").unwrap();
    let json = serde_json::to_string(&custom).unwrap();

    // Deserialize as StdDecimal
    let decimal: StdDecimal = serde_json::from_str(&json).unwrap();

    // Should be equal
    assert_eq!(decimal.to_string(), "1.5");
}

#[test]
fn test_storage_roundtrip() {
    let original = CustomDecimal::from_str("123.456789").unwrap();
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CustomDecimal = serde_json::from_str(&json).unwrap();

    // Note: Due to precision (6 decimals), we lose some precision
    // Original has more than 6 decimals, so it should be truncated to 6
    assert_eq!(original, deserialized);
}

#[test]
fn test_json_format_compact() {
    let custom = CustomDecimal::from_str("1.5").unwrap();
    let json = serde_json::to_string(&custom).unwrap();

    // Should be formatted in compact format like StdDecimal (no trailing zeros)
    assert_eq!(json, r#""1.5""#);
}

// ========== Multi-Precision Tests ==========

#[test]
fn test_different_precisions_constants() {
    // Decimal6
    assert_eq!(Decimal6::DECIMAL_PLACES, 6);
    assert_eq!(Decimal6::FRACTIONAL, 1_000_000);
    assert_eq!(Decimal6::ONE.atomics(), 1_000_000);

    // Decimal9
    assert_eq!(Decimal9::DECIMAL_PLACES, 9);
    assert_eq!(Decimal9::FRACTIONAL, 1_000_000_000);
    assert_eq!(Decimal9::ONE.atomics(), 1_000_000_000);

    // Decimal12
    assert_eq!(Decimal12::DECIMAL_PLACES, 12);
    assert_eq!(Decimal12::FRACTIONAL, 1_000_000_000_000);
    assert_eq!(Decimal12::ONE.atomics(), 1_000_000_000_000);

    // Decimal18
    assert_eq!(Decimal18::DECIMAL_PLACES, 18);
    assert_eq!(Decimal18::FRACTIONAL, 1_000_000_000_000_000_000);
    assert_eq!(Decimal18::ONE.atomics(), 1_000_000_000_000_000_000);
}

#[test]
fn test_custom_precision() {
    // Custom 4-decimal precision
    let d4 = Decimal::<4>::from_str("1.1234").unwrap();
    assert_eq!(d4.atomics(), 11234);
    assert_eq!(d4.decimal_places(), 4);

    // Custom 8-decimal precision
    let d8 = Decimal::<8>::from_str("1.12345678").unwrap();
    assert_eq!(d8.atomics(), 112345678);
    assert_eq!(d8.decimal_places(), 8);
}

#[test]
fn test_precision_conversion_scale_up() {
    let d6 = Decimal6::from_str("1.5").unwrap();

    // Scale up to 9 decimals
    let d9: Decimal9 = d6.to_precision();
    assert_eq!(d9.atomics(), 1_500_000_000);
    assert_eq!(d9.to_string(), "1.5");

    // Scale up to 18 decimals
    let d18: Decimal18 = d6.to_precision();
    assert_eq!(d18.atomics(), 1_500_000_000_000_000_000);
    assert_eq!(d18.to_string(), "1.5");
}

#[test]
fn test_precision_conversion_scale_down() {
    let d9 = Decimal9::from_str("1.123456789").unwrap();

    // Scale down to 6 decimals (truncates)
    let d6: Decimal6 = d9.to_precision();
    assert_eq!(d6.atomics(), 1_123_456);
    assert_eq!(d6.to_string(), "1.123456");
}

#[test]
fn test_precision_conversion_roundtrip() {
    let original = Decimal6::from_str("1.5").unwrap();

    // Convert to Decimal9 and back
    let d9: Decimal9 = original.to_precision();
    let back: Decimal6 = d9.to_precision();

    assert_eq!(original, back);
}

#[test]
fn test_try_to_precision() {
    let d6 = Decimal6::from_str("1.5").unwrap();

    // Should succeed
    let d9: Option<Decimal9> = d6.try_to_precision();
    assert!(d9.is_some());
    assert_eq!(d9.unwrap().atomics(), 1_500_000_000);
}

// ========== Precision Tests ==========

#[test]
fn test_precision_loss_from_decimal() {
    // StdDecimal with 18 decimals
    let decimal = StdDecimal::from_str("1.123456789012345678").unwrap();

    // Convert to Decimal6 (6 decimals)
    let d6: Decimal6 = decimal.into();
    assert_eq!(d6, Decimal6::from_str("1.123456").unwrap());

    // Convert to Decimal9 (9 decimals)
    let d9: Decimal9 = decimal.into();
    assert_eq!(d9, Decimal9::from_str("1.123456789").unwrap());
}

#[test]
fn test_one_third_precision() {
    let one = CustomDecimal::from_str("1.0").unwrap();
    let three = CustomDecimal::from_str("3.0").unwrap();

    let one_third = one / three;

    // Should be approximately 0.333333
    assert_eq!(one_third.atomics(), 333_333);

    // Multiply back should be slightly less than 1 due to truncation
    let back = one_third * three;
    assert!(back < one);
    assert_eq!(back.atomics(), 999_999);
}

#[test]
fn test_small_decimals() {
    let small = CustomDecimal::from_str("0.000001").unwrap();
    assert_eq!(small.atomics(), 1);

    let smaller = CustomDecimal::from_str("0.000000").unwrap();
    assert_eq!(smaller, CustomDecimal::ZERO);
}

#[test]
fn test_from_atomics_scaling() {
    // Scaling up (fewer decimals -> 6 decimals)
    let d1 = CustomDecimal::from_atomics(15u128, 1).unwrap();
    assert_eq!(d1.atomics(), 1_500_000); // 1.5

    // No scaling (already 6 decimals)
    let d2 = CustomDecimal::from_atomics(1_500_000u128, 6).unwrap();
    assert_eq!(d2.atomics(), 1_500_000); // 1.5

    // Scaling down (more decimals -> 6 decimals)
    let d3 = CustomDecimal::from_atomics(1_500_000_000u128, 9).unwrap();
    assert_eq!(d3.atomics(), 1_500_000); // 1.5

    // Ensure precision loss when scaling down
    let d4 = CustomDecimal::from_atomics(1_234_567_890u128, 9).unwrap();
    assert_eq!(d4.atomics(), 1_234_567); // 1.234567 (truncated from 1.234567890)
}

// ========== Edge Cases ==========

#[test]
fn test_zero_operations() {
    let zero = CustomDecimal::ZERO;
    let one = CustomDecimal::ONE;

    assert_eq!(zero + zero, zero);
    assert_eq!(zero + one, one);
    assert_eq!(one - one, zero);
    assert_eq!(zero * one, zero);
    assert!(zero.is_zero());
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_division_by_zero_panics() {
    let one = CustomDecimal::ONE;
    let zero = CustomDecimal::ZERO;
    let _ = one / zero;
}

#[test]
fn test_division_by_zero_checked() {
    let one = CustomDecimal::ONE;
    let zero = CustomDecimal::ZERO;
    assert_eq!(one.checked_div(zero), None);
}

#[test]
fn test_max_value() {
    let max = CustomDecimal::MAX;
    assert!(max > CustomDecimal::ONE);
    assert!(max > CustomDecimal::ZERO);

    // Adding to MAX should overflow
    let result = max.checked_add(CustomDecimal::ONE);
    assert_eq!(result, None);
}

#[test]
fn test_underflow() {
    let zero = CustomDecimal::ZERO;
    let one = CustomDecimal::ONE;

    // Subtracting from zero should underflow
    let result = zero.checked_sub(one);
    assert_eq!(result, None);
}

#[test]
fn test_overflow_multiplication() {
    let large = CustomDecimal::MAX;
    let two = CustomDecimal::from_str("2.0").unwrap();

    // Should overflow
    let result = large.checked_mul(two);
    assert_eq!(result, None);
}

#[test]
fn test_saturating_operations() {
    let max = CustomDecimal::MAX;
    let one = CustomDecimal::ONE;

    // Saturating add
    let result = max.saturating_add(one);
    assert_eq!(result, max);

    // Saturating sub
    let result = CustomDecimal::ZERO.saturating_sub(one);
    assert_eq!(result, CustomDecimal::ZERO);

    // Saturating mul
    let result = max.saturating_mul(CustomDecimal::from_str("2.0").unwrap());
    assert_eq!(result, max);
}

// ========== Conversion Tests ==========

#[test]
fn test_decimal_roundtrip() {
    let original = CustomDecimal::from_str("1.5").unwrap();
    let decimal: StdDecimal = original.into();
    let back: CustomDecimal = decimal.into();
    assert_eq!(original, back);
}

#[test]
fn test_uint128_conversion() {
    let uint = Uint128::new(5);
    let custom: CustomDecimal = uint.into();
    assert_eq!(custom, CustomDecimal::from_str("5.0").unwrap());

    let back = custom.to_uint_floor();
    assert_eq!(back, uint);
}

#[test]
fn test_from_u64_u128() {
    let d1 = CustomDecimal::from(42u64);
    assert_eq!(d1, CustomDecimal::from_str("42.0").unwrap());

    let d2 = CustomDecimal::from(100u128);
    assert_eq!(d2, CustomDecimal::from_str("100.0").unwrap());
}

// ========== Arithmetic Tests ==========

#[test]
fn test_complex_arithmetic() {
    let a = CustomDecimal::from_str("10.5").unwrap();
    let b = CustomDecimal::from_str("2.5").unwrap();
    let c = CustomDecimal::from_str("3.0").unwrap();

    // (10.5 + 2.5) * 3.0 = 39.0
    let result = (a + b) * c;
    assert_eq!(result, CustomDecimal::from_str("39.0").unwrap());

    // 10.5 - (2.5 * 3.0) = 3.0
    let result = a - (b * c);
    assert_eq!(result, CustomDecimal::from_str("3.0").unwrap());
}

#[test]
fn test_chained_operations() {
    let start = CustomDecimal::from_str("100.0").unwrap();
    let result = start
        .saturating_add(CustomDecimal::from_str("50.0").unwrap())
        .saturating_sub(CustomDecimal::from_str("30.0").unwrap())
        .checked_mul(CustomDecimal::from_str("2.0").unwrap())
        .unwrap();

    // (100 + 50 - 30) * 2 = 240
    assert_eq!(result, CustomDecimal::from_str("240.0").unwrap());
}

#[test]
fn test_arithmetic_different_precisions() {
    // Decimal9 arithmetic
    let a9 = Decimal9::from_str("1.5").unwrap();
    let b9 = Decimal9::from_str("2.5").unwrap();
    let sum9 = a9 + b9;
    assert_eq!(sum9, Decimal9::from_str("4.0").unwrap());

    // Decimal18 arithmetic
    let a18 = Decimal18::from_str("1.5").unwrap();
    let b18 = Decimal18::from_str("2.5").unwrap();
    let sum18 = a18 + b18;
    assert_eq!(sum18, Decimal18::from_str("4.0").unwrap());
}

// ========== Display and Parsing Tests ==========

#[test]
fn test_display_formats() {
    assert_eq!(CustomDecimal::from_str("1.5").unwrap().to_string(), "1.5");
    assert_eq!(CustomDecimal::from_str("1.0").unwrap().to_string(), "1");
    assert_eq!(
        CustomDecimal::from_str("0.1").unwrap().to_string(),
        "0.1"
    );
    assert_eq!(
        CustomDecimal::from_str("0.123456").unwrap().to_string(),
        "0.123456"
    );
    assert_eq!(
        CustomDecimal::from_str("0.100000").unwrap().to_string(),
        "0.1"
    );
}

#[test]
fn test_display_different_precisions() {
    let d9 = Decimal9::from_str("1.123456789").unwrap();
    assert_eq!(d9.to_string(), "1.123456789");

    let d18 = Decimal18::from_str("1.123456789012345678").unwrap();
    assert_eq!(d18.to_string(), "1.123456789012345678");
}

#[test]
fn test_parse_various_formats() {
    assert!(CustomDecimal::from_str("1").is_ok());
    assert!(CustomDecimal::from_str("1.0").is_ok());
    assert!(CustomDecimal::from_str("1.5").is_ok());
    assert!(CustomDecimal::from_str("0.123456").is_ok());
    assert!(CustomDecimal::from_str("123456").is_ok());
}

#[test]
fn test_parse_errors() {
    assert!(CustomDecimal::from_str("").is_err());
    assert!(CustomDecimal::from_str("abc").is_err());
    assert!(CustomDecimal::from_str("1.2.3").is_err());
    assert!(CustomDecimal::from_str("1.1234567").is_err()); // Too many decimals for Decimal6
}

#[test]
fn test_parse_precision_specific() {
    // Decimal6 can parse up to 6 decimals
    assert!(Decimal6::from_str("1.123456").is_ok());
    assert!(Decimal6::from_str("1.1234567").is_err());

    // Decimal9 can parse up to 9 decimals
    assert!(Decimal9::from_str("1.123456789").is_ok());
    assert!(Decimal9::from_str("1.1234567890").is_err());
}

// ========== Math Function Tests ==========

#[test]
fn test_sqrt_various_values() {
    assert_eq!(
        CustomDecimal::from_str("0.0").unwrap().sqrt(),
        CustomDecimal::ZERO
    );
    assert_eq!(
        CustomDecimal::from_str("1.0").unwrap().sqrt(),
        CustomDecimal::ONE
    );
    assert_eq!(
        CustomDecimal::from_str("4.0").unwrap().sqrt(),
        CustomDecimal::from_str("2.0").unwrap()
    );
    assert_eq!(
        CustomDecimal::from_str("9.0").unwrap().sqrt(),
        CustomDecimal::from_str("3.0").unwrap()
    );
}

#[test]
fn test_pow_various_exponents() {
    let base = CustomDecimal::from_str("2.0").unwrap();

    assert_eq!(base.pow(0), CustomDecimal::ONE);
    assert_eq!(base.pow(1), base);
    assert_eq!(base.pow(2), CustomDecimal::from_str("4.0").unwrap());
    assert_eq!(base.pow(3), CustomDecimal::from_str("8.0").unwrap());
    assert_eq!(base.pow(4), CustomDecimal::from_str("16.0").unwrap());
}

#[test]
fn test_floor_ceil_edge_cases() {
    let d = CustomDecimal::from_str("1.0").unwrap();
    assert_eq!(d.floor(), d);
    assert_eq!(d.ceil(), d);

    let d = CustomDecimal::from_str("1.000001").unwrap();
    assert_eq!(d.floor(), CustomDecimal::from_str("1.0").unwrap());
    assert_eq!(d.ceil(), CustomDecimal::from_str("2.0").unwrap());

    let d = CustomDecimal::from_str("1.999999").unwrap();
    assert_eq!(d.floor(), CustomDecimal::from_str("1.0").unwrap());
    assert_eq!(d.ceil(), CustomDecimal::from_str("2.0").unwrap());
}

// ========== Uint128 Operations Tests ==========

#[test]
fn test_mul_uint128_variations() {
    let dec = CustomDecimal::from_str("2.5").unwrap();
    let uint = Uint128::new(100);

    assert_eq!(dec * uint, Uint128::new(250));
    assert_eq!(uint * dec, Uint128::new(250));
    assert_eq!(&dec * uint, Uint128::new(250));
    assert_eq!(dec * &uint, Uint128::new(250));
}

#[test]
fn test_div_uint128_variations() {
    let dec = CustomDecimal::from_str("10.0").unwrap();
    let uint = Uint128::new(2);

    assert_eq!(dec / uint, CustomDecimal::from_str("5.0").unwrap());
    assert_eq!(&dec / uint, CustomDecimal::from_str("5.0").unwrap());
    assert_eq!(dec / &uint, CustomDecimal::from_str("5.0").unwrap());
}

#[test]
fn test_to_uint_rounding() {
    let d = CustomDecimal::from_str("3.7").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(3));
    assert_eq!(d.to_uint_ceil(), Uint128::new(4));

    let d = CustomDecimal::from_str("3.0").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(3));
    assert_eq!(d.to_uint_ceil(), Uint128::new(3));
}

// ========== Iterator Tests ==========

#[test]
fn test_sum_iterator() {
    let values = vec![
        CustomDecimal::from_str("1.0").unwrap(),
        CustomDecimal::from_str("2.5").unwrap(),
        CustomDecimal::from_str("3.7").unwrap(),
    ];

    let sum: CustomDecimal = values.iter().sum();
    assert_eq!(sum, CustomDecimal::from_str("7.2").unwrap());
}

#[test]
fn test_product_iterator() {
    let values = vec![
        CustomDecimal::from_str("2.0").unwrap(),
        CustomDecimal::from_str("3.0").unwrap(),
        CustomDecimal::from_str("4.0").unwrap(),
    ];

    let product: CustomDecimal = values.iter().product();
    assert_eq!(product, CustomDecimal::from_str("24.0").unwrap());
}

#[test]
fn test_empty_iterator() {
    let values: Vec<CustomDecimal> = vec![];

    let sum: CustomDecimal = values.iter().sum();
    assert_eq!(sum, CustomDecimal::ZERO);

    let product: CustomDecimal = values.iter().product();
    assert_eq!(product, CustomDecimal::ONE);
}

// ========== Cross-Precision Serialization Tests ==========

#[test]
fn test_cross_precision_serialization() {
    // Serialize Decimal6
    let d6 = Decimal6::from_str("1.5").unwrap();
    let json = serde_json::to_string(&d6).unwrap();

    // Deserialize as Decimal9
    let d9: Decimal9 = serde_json::from_str(&json).unwrap();
    assert_eq!(d9.atomics(), 1_500_000_000);

    // Deserialize as Decimal18
    let d18: Decimal18 = serde_json::from_str(&json).unwrap();
    assert_eq!(d18.atomics(), 1_500_000_000_000_000_000);
}

#[test]
fn test_std_decimal_cross_precision() {
    // Serialize StdDecimal
    let std = StdDecimal::from_str("1.5").unwrap();
    let json = serde_json::to_string(&std).unwrap();

    // Deserialize as various precisions
    let d6: Decimal6 = serde_json::from_str(&json).unwrap();
    let d9: Decimal9 = serde_json::from_str(&json).unwrap();
    let d18: Decimal18 = serde_json::from_str(&json).unwrap();

    assert_eq!(d6.to_string(), "1.5");
    assert_eq!(d9.to_string(), "1.5");
    assert_eq!(d18.to_string(), "1.5");
}

// ========== Real-World Scenario Tests ==========

#[test]
fn test_price_calculation_scenario() {
    let price_per_token = CustomDecimal::from_str("0.15").unwrap();
    let token_amount = Uint128::new(1000);
    let fee_rate = CustomDecimal::percent(3); // 3%

    let subtotal = price_per_token * token_amount;
    assert_eq!(subtotal, Uint128::new(150));

    let fee = CustomDecimal::from(subtotal) * fee_rate;
    assert_eq!(fee, CustomDecimal::from_str("4.5").unwrap());

    let total = subtotal + fee.to_uint_floor();
    assert_eq!(total, Uint128::new(154));
}

#[test]
fn test_compound_interest_scenario() {
    let principal = CustomDecimal::from_str("1000.0").unwrap();
    let rate = CustomDecimal::percent(5); // 5%
    let years = 3;

    // Compound: P * (1 + r)^t
    let multiplier = (CustomDecimal::ONE + rate).pow(years);
    let total = principal * multiplier;

    // Should be approximately 1157.625
    assert!(total > CustomDecimal::from_str("1157.0").unwrap());
    assert!(total < CustomDecimal::from_str("1158.0").unwrap());
}

#[test]
fn test_percentage_calculations() {
    let value = CustomDecimal::from_str("100.0").unwrap();

    let ten_percent = value * CustomDecimal::percent(10);
    assert_eq!(ten_percent, CustomDecimal::from_str("10.0").unwrap());

    let five_permille = value * CustomDecimal::permille(5);
    assert_eq!(five_permille, CustomDecimal::from_str("0.5").unwrap());

    let fifty_bps = value * CustomDecimal::bps(50);
    assert_eq!(fifty_bps, CustomDecimal::from_str("0.5").unwrap());
}

#[test]
fn test_multi_precision_defi_scenario() {
    // USDC-like token with 6 decimals
    let usdc_price = Decimal6::from_str("1.0").unwrap();

    // ETH-like token with 18 decimals
    let eth_price = Decimal18::from_str("2500.0").unwrap();

    // Convert to common precision for comparison
    let usdc_as_d18: Decimal18 = usdc_price.to_precision();
    let eth_to_usdc_ratio = eth_price / usdc_as_d18;

    assert_eq!(eth_to_usdc_ratio, Decimal18::from_str("2500.0").unwrap());
}
