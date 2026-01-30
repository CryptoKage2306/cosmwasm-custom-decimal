//! Comprehensive usage examples for Decimal<D> with configurable precision
//!
//! Run with: cargo run --example usage

use cosmwasm_custom_decimal::{CustomDecimal, Decimal, Decimal6, Decimal9, Decimal12, Decimal18};
use cosmwasm_std::{Decimal as StdDecimal, Uint128};
use std::str::FromStr;

fn main() {
    println!("=== Configurable Decimal Usage Examples ===\n");

    // ========== Different Precisions ==========
    println!("--- Different Precisions ---");

    let d6 = Decimal6::from_str("1.123456").unwrap();
    println!("Decimal6 (6 decimals):  {} (atomics: {})", d6, d6.atomics());

    let d9 = Decimal9::from_str("1.123456789").unwrap();
    println!("Decimal9 (9 decimals):  {} (atomics: {})", d9, d9.atomics());

    let d12 = Decimal12::from_str("1.123456789012").unwrap();
    println!("Decimal12 (12 decimals): {} (atomics: {})", d12, d12.atomics());

    let d18 = Decimal18::from_str("1.123456789012345678").unwrap();
    println!("Decimal18 (18 decimals): {} (atomics: {})", d18, d18.atomics());

    // Custom precision using the generic type directly
    let d4 = Decimal::<4>::from_str("1.1234").unwrap();
    println!("Decimal<4> (4 decimals): {} (atomics: {})", d4, d4.atomics());

    println!();

    // ========== Precision Conversion ==========
    println!("--- Precision Conversion ---");

    let d6 = Decimal6::from_str("1.5").unwrap();
    println!("Original Decimal6: {}", d6);

    // Convert to higher precision (no loss)
    let d9: Decimal9 = d6.to_precision();
    println!("Converted to Decimal9: {} (atomics: {})", d9, d9.atomics());

    let d18: Decimal18 = d6.to_precision();
    println!("Converted to Decimal18: {} (atomics: {})", d18, d18.atomics());

    // Convert back to lower precision (potential truncation)
    let d9_detailed = Decimal9::from_str("1.123456789").unwrap();
    let d6_truncated: Decimal6 = d9_detailed.to_precision();
    println!(
        "Decimal9 {} -> Decimal6 {} (truncated)",
        d9_detailed, d6_truncated
    );

    println!();

    // ========== Type Safety ==========
    println!("--- Type Safety ---");

    let a6 = Decimal6::from_str("1.5").unwrap();
    let b6 = Decimal6::from_str("2.5").unwrap();

    // Same precision - works fine
    let sum6 = a6 + b6;
    println!("Decimal6: {} + {} = {}", a6, b6, sum6);

    let a9 = Decimal9::from_str("1.5").unwrap();
    let b9 = Decimal9::from_str("2.5").unwrap();

    // Same precision - works fine
    let sum9 = a9 + b9;
    println!("Decimal9: {} + {} = {}", a9, b9, sum9);

    // Different precisions - must convert first!
    // This would NOT compile: let _ = a6 + a9;
    let a6_as_d9: Decimal9 = a6.to_precision();
    let mixed_sum = a6_as_d9 + a9;
    println!(
        "Mixed (converted): {} + {} = {}",
        a6_as_d9, a9, mixed_sum
    );

    println!();

    // ========== Construction Methods ==========
    println!("--- Construction Methods (using Decimal6) ---");

    let d1 = CustomDecimal::raw(1_500_000);
    println!("raw(1_500_000) = {}", d1); // 1.5

    let d2 = CustomDecimal::from_str("1.5").unwrap();
    println!("from_str(\"1.5\") = {}", d2); // 1.5

    let d3 = CustomDecimal::from_atomics(15u128, 1).unwrap();
    println!("from_atomics(15, 1) = {}", d3); // 1.5

    let d4 = CustomDecimal::percent(50);
    println!("percent(50) = {}", d4); // 0.5

    let d5 = CustomDecimal::permille(125);
    println!("permille(125) = {}", d5); // 0.125

    let d6 = CustomDecimal::bps(50);
    println!("bps(50) = {}", d6); // 0.005

    let d7 = CustomDecimal::from_ratio(3u128, 2u128);
    println!("from_ratio(3, 2) = {}", d7); // 1.5

    println!();

    // ========== Constants ==========
    println!("--- Constants (different precisions) ---");
    println!("Decimal6::ZERO = {}", Decimal6::ZERO);
    println!("Decimal6::ONE = {} (atomics: {})", Decimal6::ONE, Decimal6::ONE.atomics());
    println!("Decimal6::DECIMAL_PLACES = {}", Decimal6::DECIMAL_PLACES);
    println!("Decimal6::FRACTIONAL = {}", Decimal6::FRACTIONAL);
    println!();
    println!("Decimal9::ONE = {} (atomics: {})", Decimal9::ONE, Decimal9::ONE.atomics());
    println!("Decimal9::DECIMAL_PLACES = {}", Decimal9::DECIMAL_PLACES);
    println!("Decimal9::FRACTIONAL = {}", Decimal9::FRACTIONAL);
    println!();
    println!("Decimal18::ONE = {} (atomics: {})", Decimal18::ONE, Decimal18::ONE.atomics());
    println!("Decimal18::DECIMAL_PLACES = {}", Decimal18::DECIMAL_PLACES);
    println!("Decimal18::FRACTIONAL = {}", Decimal18::FRACTIONAL);
    println!();

    // ========== Arithmetic ==========
    println!("--- Arithmetic Operations ---");

    let a = CustomDecimal::from_str("1.5").unwrap();
    let b = CustomDecimal::from_str("2.0").unwrap();

    println!("{} + {} = {}", a, b, a + b); // 3.5
    println!("{} - {} = {}", b, a, b - a); // 0.5
    println!("{} * {} = {}", a, b, a * b); // 3.0
    println!("{} / {} = {}", b, a, b / a); // 1.333333
    println!("{} % {} = {}", b, a, b % a); // 0.5

    println!();

    // ========== Checked Operations ==========
    println!("--- Checked Operations ---");

    let c = CustomDecimal::from_str("100.0").unwrap();
    let d = CustomDecimal::from_str("50.0").unwrap();

    println!("checked_add({}, {}) = {:?}", c, d, c.checked_add(d));
    println!("checked_sub({}, {}) = {:?}", c, d, c.checked_sub(d));
    println!("checked_mul({}, {}) = {:?}", c, d, c.checked_mul(d));
    println!("checked_div({}, {}) = {:?}", c, d, c.checked_div(d));

    let zero = CustomDecimal::ZERO;
    println!(
        "checked_div({}, {}) = {:?} (division by zero)",
        c,
        zero,
        c.checked_div(zero)
    );

    println!();

    // ========== Rounding & Math ==========
    println!("--- Rounding & Math ---");

    let num = CustomDecimal::from_str("1.7").unwrap();
    println!("floor({}) = {}", num, num.floor()); // 1.0
    println!("ceil({}) = {}", num, num.ceil()); // 2.0

    let sqrt_input = CustomDecimal::from_str("9.0").unwrap();
    println!("sqrt({}) = {}", sqrt_input, sqrt_input.sqrt()); // 3.0

    let base = CustomDecimal::from_str("2.0").unwrap();
    println!("pow({}, 3) = {}", base, base.pow(3)); // 8.0

    println!();

    // ========== Operations with Uint128 ==========
    println!("--- Operations with Uint128 ---");

    let decimal = CustomDecimal::from_str("2.5").unwrap();
    let uint = Uint128::new(1000);

    println!("{} * {} = {}", decimal, uint, decimal * uint); // 2500
    println!("{} * {} = {}", uint, decimal, uint * decimal); // 2500
    println!("{} / {} = {}", decimal, Uint128::new(2), decimal / Uint128::new(2)); // 1.25

    println!();

    // ========== Decimal Interop (cosmwasm_std::Decimal) ==========
    println!("--- cosmwasm_std::Decimal Interop ---");

    let custom = CustomDecimal::from_str("1.5").unwrap();
    println!("CustomDecimal (6 decimals): {}", custom);

    let std_decimal: StdDecimal = custom.into();
    println!("Converted to StdDecimal (18 decimals): {}", std_decimal);

    let back: CustomDecimal = std_decimal.into();
    println!("Converted back to CustomDecimal: {}", back);

    // Same with Decimal9
    let d9 = Decimal9::from_str("1.123456789").unwrap();
    let std_from_d9: StdDecimal = d9.into();
    println!("Decimal9 {} -> StdDecimal {}", d9, std_from_d9);

    println!();

    // ========== Storage Compatibility ==========
    println!("--- Storage Compatibility (JSON Serialization) ---");

    let custom = CustomDecimal::from_str("1.5").unwrap();
    let json = serde_json::to_string(&custom).unwrap();
    println!("Serialized CustomDecimal: {}", json);

    // Show compatibility with StdDecimal
    let std_decimal = StdDecimal::from_str("1.5").unwrap();
    let std_json = serde_json::to_string(&std_decimal).unwrap();
    println!("Serialized StdDecimal: {}", std_json);

    println!(
        "Are they equal? {}",
        if json == std_json { "YES" } else { "NO" }
    );

    // Deserialize StdDecimal JSON as CustomDecimal
    let loaded: CustomDecimal = serde_json::from_str(&std_json).unwrap();
    println!("Loaded from StdDecimal JSON: {}", loaded);

    // Cross-precision serialization
    println!();
    println!("Cross-precision serialization:");
    let d6 = Decimal6::from_str("1.5").unwrap();
    let d6_json = serde_json::to_string(&d6).unwrap();
    println!("Decimal6 serialized: {}", d6_json);

    let d18_from_d6_json: Decimal18 = serde_json::from_str(&d6_json).unwrap();
    println!(
        "Deserialized as Decimal18: {} (atomics: {})",
        d18_from_d6_json,
        d18_from_d6_json.atomics()
    );

    println!();

    // ========== Iterator Operations ==========
    println!("--- Iterator Operations ---");

    let values = vec![
        CustomDecimal::from_str("1.0").unwrap(),
        CustomDecimal::from_str("2.0").unwrap(),
        CustomDecimal::from_str("3.0").unwrap(),
    ];

    let sum: CustomDecimal = values.iter().sum();
    println!("Sum of [1.0, 2.0, 3.0] = {}", sum); // 6.0

    let product: CustomDecimal = values.iter().product();
    println!("Product of [1.0, 2.0, 3.0] = {}", product); // 6.0

    println!();

    // ========== Practical Example: Token Price with Different Precisions ==========
    println!("--- Practical Example: Token Pricing ---");

    // Some tokens use different decimal precisions
    // USDC-like (6 decimals)
    let usdc_price = Decimal6::from_str("1.0").unwrap();
    let usdc_amount = Uint128::new(1_000_000); // 1 USDC in micro units

    // ETH-like (18 decimals)
    let eth_price = Decimal18::from_str("2500.0").unwrap();
    let eth_amount_wei = Uint128::new(1_000_000_000_000_000_000u128); // 1 ETH in wei

    println!("USDC price: {} (6 decimals)", usdc_price);
    println!("ETH price: {} (18 decimals)", eth_price);

    // Calculate values (in their native precision)
    let usdc_value = usdc_price * usdc_amount;
    let eth_value = eth_price * eth_amount_wei;

    println!("1 USDC value: {} (in micro units)", usdc_value);
    println!("1 ETH value: {} (in wei)", eth_value);

    println!();

    // ========== Practical Example: Interest Calculation ==========
    println!("--- Practical Example: Interest Calculation ---");

    let principal = CustomDecimal::from_str("1000.0").unwrap();
    let annual_rate = CustomDecimal::percent(5); // 5% per year
    let years = 3;

    println!("Principal: {}", principal);
    println!("Annual rate: {} ({}%)", annual_rate, 5);
    println!("Years: {}", years);

    // Simple interest: P * r * t
    let simple_interest = principal * annual_rate * CustomDecimal::from(years as u64);
    println!("Simple interest: {}", simple_interest);
    println!("Total (simple): {}", principal + simple_interest);

    // Compound interest: P * (1 + r)^t
    let compound_multiplier = (CustomDecimal::ONE + annual_rate).pow(years);
    let compound_total = principal * compound_multiplier;
    let compound_interest = compound_total - principal;
    println!("Compound interest: {}", compound_interest);
    println!("Total (compound): {}", compound_total);

    println!();
    println!("=== Examples Complete ===");
}
