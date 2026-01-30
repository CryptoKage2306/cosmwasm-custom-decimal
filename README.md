# CosmWasm Custom Decimal

A production-ready Rust library for CosmWasm smart contracts implementing a `Decimal<D>` type with **configurable decimal precision** via const generics, while maintaining full compatibility with `cosmwasm_std::Decimal`.

## Features

- **Configurable Precision**: Define any decimal precision using const generics: `Decimal<6>`, `Decimal<9>`, `Decimal<18>`, etc.
- **Type Safety**: Different precisions are distinct types - prevents accidental mixing at compile time
- **Transparent Storage**: Serializes identically to `cosmwasm_std::Decimal` for seamless migration
- **Full API Parity**: Complete compatibility with `cosmwasm_std::Decimal` API
- **Safe Math**: Overflow-protected operations using `Uint256` intermediates
- **Zero-Cost Conversions**: Efficient conversion between different precisions

## Why Configurable Precision?

Different use cases require different precision levels:

| Use Case | Recommended Precision | Type |
|----------|----------------------|------|
| Stablecoin prices (USDC-like) | 6 decimals | `Decimal6` |
| High-precision DeFi | 9 decimals | `Decimal9` |
| Scientific calculations | 12 decimals | `Decimal12` |
| Full Decimal compatibility | 18 decimals | `Decimal18` |
| Custom needs | Any | `Decimal<N>` |

## Installation

```toml
[dependencies]
cosmwasm-custom-decimal = "0.1"
cosmwasm-std = "2.0"
```

## Quick Start

```rust
use cosmwasm_custom_decimal::{Decimal, Decimal6, Decimal9, Decimal18, CustomDecimal};
use std::str::FromStr;

// Use predefined type aliases
let d6 = Decimal6::from_str("1.5").unwrap();        // 6 decimal places
let d9 = Decimal9::from_str("1.123456789").unwrap(); // 9 decimal places
let d18 = Decimal18::from_str("1.5").unwrap();       // 18 decimal places

// Or use the generic type directly for custom precision
let d4 = Decimal::<4>::from_str("1.1234").unwrap();  // 4 decimal places

// CustomDecimal is an alias for Decimal<6> (backward compatible)
let custom = CustomDecimal::from_str("1.5").unwrap();

// Arithmetic works within same precision
let sum = d6 + Decimal6::from_str("2.5").unwrap(); // 4.0

// Convert between precisions
let d9_from_d6: Decimal9 = d6.to_precision(); // 1.500000000
```

## Type Safety

Different precisions are different types and cannot be mixed in operations:

```rust
use cosmwasm_custom_decimal::{Decimal6, Decimal9};

let d6 = Decimal6::ONE;
let d9 = Decimal9::ONE;

// This will NOT compile - type mismatch!
// let sum = d6 + d9;

// You must explicitly convert first:
let d6_as_d9: Decimal9 = d6.to_precision();
let sum = d6_as_d9 + d9; // Works!
```

## Precision Conversion

```rust
use cosmwasm_custom_decimal::{Decimal6, Decimal9, Decimal18};

let d6 = Decimal6::from_str("1.123456").unwrap();

// Scale up (no precision loss)
let d9: Decimal9 = d6.to_precision();
let d18: Decimal18 = d6.to_precision();

// Scale down (truncates extra decimals)
let d9_detailed = Decimal9::from_str("1.123456789").unwrap();
let d6_truncated: Decimal6 = d9_detailed.to_precision(); // 1.123456

// Safe conversion with overflow checking
let maybe_d9: Option<Decimal9> = d6.try_to_precision();
```

## API Documentation

### Type Aliases

```rust
pub type CustomDecimal = Decimal<6>;  // Backward compatible default
pub type Decimal6 = Decimal<6>;       // 6 decimal places
pub type Decimal9 = Decimal<9>;       // 9 decimal places
pub type Decimal12 = Decimal<12>;     // 12 decimal places
pub type Decimal18 = Decimal<18>;     // 18 decimal places (matches cosmwasm_std::Decimal)
```

### Constants

Each `Decimal<D>` type has these associated constants:

```rust
Decimal6::ZERO            // 0
Decimal6::ONE             // 1.0 (stored as 1_000_000)
Decimal6::MAX             // Maximum value
Decimal6::DECIMAL_PLACES  // 6
Decimal6::FRACTIONAL      // 1_000_000 (10^6)

Decimal9::ONE             // 1.0 (stored as 1_000_000_000)
Decimal9::FRACTIONAL      // 1_000_000_000 (10^9)
```

### Construction

```rust
// From raw atomic value
Decimal6::raw(1_500_000)  // 1.5
Decimal9::raw(1_500_000_000)  // 1.5

// From string
Decimal6::from_str("1.5").unwrap()
Decimal9::from_str("1.123456789").unwrap()

// From atomics with scaling
Decimal6::from_atomics(15u128, 1).unwrap()  // 1.5

// From percentage (0-100)
Decimal6::percent(50)  // 0.5

// From permille (0-1000)
Decimal6::permille(125)  // 0.125

// From basis points (0-10000)
Decimal6::bps(50)  // 0.005

// From ratio
Decimal6::from_ratio(3u128, 2u128)  // 1.5
```

### Arithmetic

```rust
let a = Decimal6::from_str("1.5").unwrap();
let b = Decimal6::from_str("2.0").unwrap();

// Basic operations (same precision only)
let sum = a + b;
let diff = a - b;
let product = a * b;
let quotient = a / b;

// Checked operations (return Option)
let safe_sum = a.checked_add(b);
let safe_product = a.checked_mul(b);

// Saturating operations
let saturated = a.saturating_add(Decimal6::MAX);

// Operations with Uint128
let amount = Uint128::new(1000);
let result = a * amount; // Decimal6 * Uint128 = Uint128
```

### Precision Conversion

```rust
let d6 = Decimal6::from_str("1.5").unwrap();

// Convert to different precision
let d9: Decimal9 = d6.to_precision();
let d18: Decimal18 = d6.to_precision();

// Safe conversion (returns Option)
let maybe_d9: Option<Decimal9> = d6.try_to_precision();

// Convert back (may truncate)
let d6_back: Decimal6 = d9.to_precision();
```

### Utilities

```rust
let val = Decimal6::from_str("1.7").unwrap();

// Rounding
val.floor()  // 1.0
val.ceil()   // 2.0

// Math operations
val.sqrt()
val.pow(2)  // 2.89

// Comparisons
val.min(Decimal6::ONE)
val.max(Decimal6::ZERO)
val.abs_diff(Decimal6::ONE)

// Conversions
val.to_uint_floor()  // Uint128(1)
val.to_uint_ceil()   // Uint128(2)

// Accessors
val.atomics()         // Returns raw u128
val.decimal_places()  // Returns 6 (or D for Decimal<D>)
val.is_zero()
```

### Conversions to/from cosmwasm_std::Decimal

```rust
use cosmwasm_std::Decimal as StdDecimal;

// From StdDecimal (truncates precision)
let std_decimal = StdDecimal::from_str("1.500000000000000000").unwrap();
let d6: Decimal6 = std_decimal.into();

// To StdDecimal (scales up)
let std_decimal: StdDecimal = d6.into();

// Works with any precision
let d9 = Decimal9::from_str("1.5").unwrap();
let std_from_d9: StdDecimal = d9.into();
```

## Storage Compatibility

`Decimal<D>` serializes identically to `cosmwasm_std::Decimal`, enabling transparent storage:

```rust
use cosmwasm_std::Decimal as StdDecimal;
use cw_storage_plus::Item;

// Original contract with StdDecimal
const PRICE: Item<StdDecimal> = Item::new("price");

// Can be read as Decimal6 without migration!
const PRICE_CUSTOM: Item<Decimal6> = Item::new("price");

// JSON format is compatible
let d6 = Decimal6::from_str("1.5").unwrap();
let std = StdDecimal::from_str("1.5").unwrap();

assert_eq!(
    serde_json::to_string(&d6).unwrap(),
    serde_json::to_string(&std).unwrap()
);  // Both serialize to "1.5"
```

### Cross-Precision Serialization

```rust
// Serialize Decimal6
let d6 = Decimal6::from_str("1.5").unwrap();
let json = serde_json::to_string(&d6).unwrap();  // "1.5"

// Deserialize as Decimal18 (no precision loss)
let d18: Decimal18 = serde_json::from_str(&json).unwrap();

// Deserialize as Decimal6 from 18-decimal format
let json_18 = "\"1.500000000000000000\"";
let d6_loaded: Decimal6 = serde_json::from_str(json_18).unwrap();
```

## Implementation Details

### Const Generic Implementation

The `Decimal<D>` type uses Rust's const generics to compute values at compile time:

```rust
pub struct Decimal<const D: u32>(Uint128);

impl<const D: u32> Decimal<D> {
    pub const FRACTIONAL: u128 = pow10(D);  // Computed at compile time
    pub const ONE: Self = Self(Uint128::new(pow10(D)));
}
```

### Overflow Protection

All multiplication and division operations use `Uint256` intermediates:

```rust
let result = Uint256::from(a.0)
    .checked_mul(Uint256::from(b.0))
    .checked_div(Uint256::from(Self::FRACTIONAL));
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run example
cargo run --example usage
```

## Examples

See [examples/usage.rs](examples/usage.rs) for comprehensive examples including:
- Different precision types
- Precision conversion
- Type safety demonstration
- Arithmetic operations
- cosmwasm_std::Decimal interop
- Storage compatibility
- Real-world use cases

## Migration Guide

### From Previous Version (non-generic)

If you're upgrading from a non-generic `CustomDecimal`:

```rust
// Old code (still works - CustomDecimal is now an alias)
use cosmwasm_custom_decimal::CustomDecimal;
let d = CustomDecimal::from_str("1.5").unwrap();

// New explicit way
use cosmwasm_custom_decimal::Decimal6;
let d = Decimal6::from_str("1.5").unwrap();

// Or use the generic type
use cosmwasm_custom_decimal::Decimal;
let d = Decimal::<6>::from_str("1.5").unwrap();
```

All existing code using `CustomDecimal` will continue to work without changes.

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT)

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- New features include tests
- Documentation is updated
- Code follows Rust best practices

## Changelog

### 0.2.0
- **Breaking**: Refactored to use const generics for configurable precision
- Added `Decimal<D>` generic type
- Added type aliases: `Decimal6`, `Decimal9`, `Decimal12`, `Decimal18`
- Added `to_precision()` and `try_to_precision()` for precision conversion
- `CustomDecimal` is now a type alias for `Decimal<6>` (backward compatible)
- Added cross-precision serialization support

### 0.1.0
- Initial release
- Full Decimal API parity
- Storage compatibility
- Comprehensive test suite
