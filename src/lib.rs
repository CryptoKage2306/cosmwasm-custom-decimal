//! # CosmWasm Custom Decimal
//!
//! A production-ready decimal type for CosmWasm smart contracts with configurable decimal places,
//! while maintaining full compatibility with `cosmwasm_std::Decimal`.
//!
//! ## Features
//!
//! - **Configurable Precision**: Use any decimal precision via const generics (e.g., `Decimal<6>`, `Decimal<9>`)
//! - **Transparent Storage**: Serializes identically to `cosmwasm_std::Decimal` for seamless migration
//! - **Full API Parity**: Complete compatibility with `cosmwasm_std::Decimal` API
//! - **Safe Math**: Overflow-protected operations using `Uint256` intermediates
//! - **Type Safety**: Different precisions are distinct types, preventing accidental mixing
//!
//! ## Quick Start
//!
//! ```
//! use cosmwasm_custom_decimal::{CustomDecimal, Decimal6, Decimal9, Decimal18};
//! use std::str::FromStr;
//!
//! // Use the default 6-decimal type
//! let price = CustomDecimal::from_str("1.5").unwrap();
//!
//! // Or use specific precisions
//! let d6 = Decimal6::from_str("1.5").unwrap();
//! let d9 = Decimal9::from_str("1.5").unwrap();
//! let d18 = Decimal18::from_str("1.5").unwrap();
//!
//! // Arithmetic works within same precision
//! let total = price * CustomDecimal::from_str("2.0").unwrap();
//!
//! // Convert between precisions
//! let converted: Decimal9 = d6.to_precision();
//! ```

use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{Decimal as StdDecimal, Decimal256, Uint128, Uint256};
use std::fmt;
use std::iter::{Product, Sum};
use std::str::FromStr;

mod error;
mod ops;
mod serde_impl;

pub use error::CustomDecimalError;

// ========== Const Helper Functions ==========

/// Compute 10^exp at compile time
pub const fn pow10(exp: u32) -> u128 {
    let mut result: u128 = 1;
    let mut i = 0;
    while i < exp {
        result *= 10;
        i += 1;
    }
    result
}

/// Compute the scale factor to convert from D decimals to 18 decimals
pub const fn scale_factor_to_18<const D: u32>() -> u128 {
    if D >= 18 {
        1
    } else {
        pow10(18 - D)
    }
}

/// Compute the scale factor to convert from 18 decimals to D decimals
pub const fn scale_factor_from_18<const D: u32>() -> u128 {
    if D >= 18 {
        1
    } else {
        pow10(18 - D)
    }
}

// ========== Legacy Constants (for backward compatibility) ==========

/// Number of decimal places for CustomDecimal (default: 6)
pub const CUSTOM_DECIMALS: u32 = 6;

/// Fractional multiplier for 6 decimals: 10^6
pub const CUSTOM_DECIMAL_FRACTIONAL: u128 = 1_000_000;

/// Scale factor to convert between 6 and 18 decimal places: 10^12
pub const SCALE_FACTOR: u128 = 1_000_000_000_000;

// ========== Type Aliases ==========

/// A decimal with 6 decimal places (default, backward compatible)
pub type CustomDecimal = Decimal<6>;

/// A decimal with 6 decimal places
pub type Decimal6 = Decimal<6>;

/// A decimal with 9 decimal places
pub type Decimal9 = Decimal<9>;

/// A decimal with 12 decimal places
pub type Decimal12 = Decimal<12>;

/// A decimal with 18 decimal places (matches cosmwasm_std::Decimal)
pub type Decimal18 = Decimal<18>;

/// A fixed-point decimal with configurable decimal places.
///
/// The const generic parameter `D` specifies the number of decimal places.
/// Internally stores values as `Uint128` atomics where 1.0 = 10^D.
///
/// When serialized, it formats as 18-decimal strings (e.g., "1.500000000000000000")
/// for transparent compatibility with `cosmwasm_std::Decimal` storage.
///
/// # Type Safety
///
/// Different precisions are different types and cannot be mixed in operations:
/// ```compile_fail
/// use cosmwasm_custom_decimal::{Decimal6, Decimal9};
/// let d6 = Decimal6::ONE;
/// let d9 = Decimal9::ONE;
/// let _ = d6 + d9; // Compile error: mismatched types
/// ```
///
/// Use `to_precision()` to convert between precisions:
/// ```
/// use cosmwasm_custom_decimal::{Decimal6, Decimal9};
/// let d6 = Decimal6::ONE;
/// let d9: Decimal9 = d6.to_precision();
/// ```
///
/// # Examples
///
/// ```
/// use cosmwasm_custom_decimal::Decimal;
/// use cosmwasm_std::Uint128;
/// use std::str::FromStr;
///
/// // 6-decimal precision
/// let a = Decimal::<6>::from_str("1.5").unwrap();
/// let b = Decimal::<6>::percent(50); // 0.5
/// let sum = a + b; // 2.0
///
/// // 9-decimal precision
/// let c = Decimal::<9>::from_str("1.123456789").unwrap();
/// ```
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
#[schemars(transparent)]
pub struct Decimal<const D: u32>(pub(crate) Uint128);

impl<const D: u32> Decimal<D> {
    // ========== Constants ==========

    /// The fractional multiplier: 10^D
    pub const FRACTIONAL: u128 = pow10(D);

    /// Zero decimal value
    pub const ZERO: Self = Self(Uint128::zero());

    /// One decimal value (1.0)
    pub const ONE: Self = Self(Uint128::new(pow10(D)));

    /// Maximum decimal value
    pub const MAX: Self = Self(Uint128::MAX);

    /// Number of decimal places
    pub const DECIMAL_PLACES: u32 = D;

    // ========== Construction ==========

    /// Create a Decimal from raw atomic units.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// let d = Decimal::<6>::raw(1_500_000); // 1.5
    /// ```
    pub const fn raw(atomics: u128) -> Self {
        Self(Uint128::new(atomics))
    }

    /// Create from atomics with specified decimal places, scaling as needed.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// let d = Decimal::<6>::from_atomics(15u128, 1).unwrap(); // 1.5
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Uint128>,
        decimal_places: u32,
    ) -> Result<Self, CustomDecimalError> {
        let atomics = atomics.into();

        Ok(match decimal_places.cmp(&D) {
            std::cmp::Ordering::Less => {
                // Scale up
                let scale = pow10(D - decimal_places);
                Self(
                    atomics
                        .checked_mul(Uint128::from(scale))
                        .map_err(|_| CustomDecimalError::Overflow)?,
                )
            }
            std::cmp::Ordering::Equal => Self(atomics),
            std::cmp::Ordering::Greater => {
                // Scale down
                let scale = pow10(decimal_places - D);
                Self(atomics.checked_div(Uint128::from(scale)).unwrap())
            }
        })
    }

    /// Create from a percentage value (0-100).
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// let d = Decimal::<6>::percent(50); // 0.5
    /// ```
    pub fn percent(x: u64) -> Self {
        Self(Uint128::from(x) * Uint128::from(Self::FRACTIONAL / 100))
    }

    /// Create from a permille value (0-1000).
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// let d = Decimal::<6>::permille(125); // 0.125
    /// ```
    pub fn permille(x: u64) -> Self {
        Self(Uint128::from(x) * Uint128::from(Self::FRACTIONAL / 1000))
    }

    /// Create from basis points (0-10000).
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// let d = Decimal::<6>::bps(50); // 0.005
    /// ```
    pub fn bps(x: u64) -> Self {
        Self(Uint128::from(x) * Uint128::from(Self::FRACTIONAL / 10000))
    }

    /// Create from a ratio of two values.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// let d = Decimal::<6>::from_ratio(3u128, 2u128); // 1.5
    /// ```
    pub fn from_ratio(numerator: impl Into<Uint128>, denominator: impl Into<Uint128>) -> Self {
        let numerator: Uint128 = numerator.into();
        let denominator: Uint128 = denominator.into();

        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }

        // Use Uint256 to prevent overflow
        let result = Uint256::from(numerator)
            .checked_mul(Uint256::from(Self::FRACTIONAL))
            .unwrap()
            .checked_div(Uint256::from(denominator))
            .unwrap();

        Self(Uint128::try_from(result).expect("ratio overflow"))
    }

    // ========== Accessors ==========

    /// Returns the raw atomic value.
    pub const fn atomics(&self) -> u128 {
        self.0.u128()
    }

    /// Returns the number of decimal places.
    pub const fn decimal_places(&self) -> u32 {
        D
    }

    /// Returns true if the value is zero.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    // ========== Precision Conversion ==========

    /// Convert to a different decimal precision.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::{Decimal6, Decimal9};
    /// use std::str::FromStr;
    /// let d6 = Decimal6::from_str("1.5").unwrap();
    /// let d9: Decimal9 = d6.to_precision();
    /// ```
    pub fn to_precision<const D2: u32>(&self) -> Decimal<D2> {
        if D == D2 {
            // Same precision, just copy
            Decimal(self.0)
        } else if D < D2 {
            // Scale up
            let scale = pow10(D2 - D);
            Decimal(self.0.checked_mul(Uint128::from(scale)).expect("precision conversion overflow"))
        } else {
            // Scale down (D > D2)
            let scale = pow10(D - D2);
            Decimal(self.0 / Uint128::from(scale))
        }
    }

    /// Try to convert to a different decimal precision, returning None on overflow.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::{Decimal6, Decimal9};
    /// use std::str::FromStr;
    /// let d6 = Decimal6::from_str("1.5").unwrap();
    /// let d9: Option<Decimal9> = d6.try_to_precision();
    /// ```
    pub fn try_to_precision<const D2: u32>(&self) -> Option<Decimal<D2>> {
        if D == D2 {
            Some(Decimal(self.0))
        } else if D < D2 {
            let scale = pow10(D2 - D);
            self.0.checked_mul(Uint128::from(scale)).ok().map(Decimal)
        } else {
            let scale = pow10(D - D2);
            Some(Decimal(self.0 / Uint128::from(scale)))
        }
    }

    // ========== Checked Operations ==========

    /// Checked addition. Returns `None` on overflow.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).ok().map(Self)
    }

    /// Checked subtraction. Returns `None` on underflow.
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).ok().map(Self)
    }

    /// Checked multiplication. Returns `None` on overflow.
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let result = Uint256::from(self.0)
            .checked_mul(Uint256::from(other.0)).ok()?
            .checked_div(Uint256::from(Self::FRACTIONAL)).ok()?;

        Uint128::try_from(result).ok().map(Self)
    }

    /// Checked division. Returns `None` on division by zero or overflow.
    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other.0.is_zero() {
            return None;
        }

        let numerator = Uint256::from(self.0)
            .checked_mul(Uint256::from(Self::FRACTIONAL)).ok()?;
        let result = numerator.checked_div(Uint256::from(other.0)).ok()?;

        Uint128::try_from(result).ok().map(Self)
    }

    /// Checked remainder. Returns `None` on division by zero.
    pub fn checked_rem(self, other: Self) -> Option<Self> {
        self.0.checked_rem(other.0).ok().map(Self)
    }

    /// Checked power. Returns `None` on overflow.
    pub fn checked_pow(self, exp: u32) -> Option<Self> {
        // Special cases
        if exp == 0 {
            return Some(Self::ONE);
        }
        if exp == 1 {
            return Some(self);
        }
        if self.is_zero() {
            return Some(Self::ZERO);
        }

        // Use repeated multiplication with overflow checking
        let mut result = self;
        for _ in 1..exp {
            result = result.checked_mul(self)?;
        }
        Some(result)
    }

    // ========== Saturating Operations ==========

    /// Saturating addition. Returns `MAX` on overflow.
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction. Returns `ZERO` on underflow.
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Saturating multiplication. Returns `MAX` on overflow.
    pub fn saturating_mul(self, other: Self) -> Self {
        match self.checked_mul(other) {
            Some(result) => result,
            None => Self::MAX,
        }
    }

    // ========== Rounding & Math ==========

    /// Returns the largest integer less than or equal to this value.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// use std::str::FromStr;
    /// let d = Decimal::<6>::from_str("1.7").unwrap();
    /// assert_eq!(d.floor(), Decimal::<6>::from_str("1.0").unwrap());
    /// ```
    pub fn floor(self) -> Self {
        Self(
            (self.0.u128() / Self::FRACTIONAL * Self::FRACTIONAL).into(),
        )
    }

    /// Returns the smallest integer greater than or equal to this value.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// use std::str::FromStr;
    /// let d = Decimal::<6>::from_str("1.1").unwrap();
    /// assert_eq!(d.ceil(), Decimal::<6>::from_str("2.0").unwrap());
    /// ```
    pub fn ceil(self) -> Self {
        let floor = self.floor();
        if self == floor {
            floor
        } else {
            floor + Self::ONE
        }
    }

    /// Square root using Decimal's sqrt internally (converts to/from).
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// use std::str::FromStr;
    /// let d = Decimal::<6>::from_str("4.0").unwrap();
    /// assert_eq!(d.sqrt(), Decimal::<6>::from_str("2.0").unwrap());
    /// ```
    pub fn sqrt(self) -> Self {
        // Convert to cosmwasm_std::Decimal (18 decimals), use its sqrt, convert back
        let decimal: StdDecimal = self.into();
        let sqrt_decimal = decimal.sqrt();
        sqrt_decimal.into()
    }

    /// Power function.
    ///
    /// # Example
    /// ```
    /// use cosmwasm_custom_decimal::Decimal;
    /// use std::str::FromStr;
    /// let d = Decimal::<6>::from_str("2.0").unwrap();
    /// assert_eq!(d.pow(3), Decimal::<6>::from_str("8.0").unwrap());
    /// ```
    pub fn pow(self, exp: u32) -> Self {
        self.checked_pow(exp).expect("overflow in pow")
    }

    // ========== Comparisons ==========

    /// Returns the minimum of two values.
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    /// Returns the maximum of two values.
    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }

    /// Returns the absolute difference between two values.
    pub fn abs_diff(self, other: Self) -> Self {
        if self > other {
            self - other
        } else {
            other - self
        }
    }

    // ========== Conversions to Uint128 ==========

    /// Convert to `Uint128` by flooring.
    pub fn to_uint_floor(self) -> Uint128 {
        self.0 / Uint128::from(Self::FRACTIONAL)
    }

    /// Convert to `Uint128` by ceiling.
    pub fn to_uint_ceil(self) -> Uint128 {
        self.ceil().to_uint_floor()
    }

    /// Multiply by `Uint128` and floor the result.
    pub fn mul_uint_floor(self, rhs: Uint128) -> Uint128 {
        (self * rhs).checked_div(Uint128::one()).unwrap()
    }

    /// Multiply by `Uint128` and ceil the result.
    pub fn mul_uint_ceil(self, rhs: Uint128) -> Uint128 {
        let product = self * rhs;
        let floored = product / Uint128::one();
        if product % Uint128::one() == Uint128::zero() {
            floored
        } else {
            floored + Uint128::one()
        }
    }
}

// ========== Type Conversions ==========

/// From Uint128 (treats as integer, e.g., 5 -> 5.0)
impl<const D: u32> From<Uint128> for Decimal<D> {
    fn from(value: Uint128) -> Self {
        Self(value * Uint128::from(Self::FRACTIONAL))
    }
}

/// From u128 (treats as integer)
impl<const D: u32> From<u128> for Decimal<D> {
    fn from(value: u128) -> Self {
        Self::from(Uint128::from(value))
    }
}

/// From u64 (treats as integer)
impl<const D: u32> From<u64> for Decimal<D> {
    fn from(value: u64) -> Self {
        Self::from(Uint128::from(value))
    }
}

/// Convert from cosmwasm_std::Decimal (truncates precision from 18 to D decimals)
impl<const D: u32> From<StdDecimal> for Decimal<D> {
    fn from(decimal: StdDecimal) -> Self {
        // StdDecimal stores as Uint128 with 18 decimals
        let atomics = decimal.atomics();
        if D >= 18 {
            // Scale up (rare case)
            let scale = pow10(D - 18);
            Self(atomics * Uint128::from(scale))
        } else {
            // Scale down (common case)
            let scale = pow10(18 - D);
            Self(atomics / Uint128::from(scale))
        }
    }
}

/// Convert to cosmwasm_std::Decimal (scales from D to 18 decimals)
impl<const D: u32> From<Decimal<D>> for StdDecimal {
    fn from(custom: Decimal<D>) -> Self {
        if D >= 18 {
            // Scale down (rare case)
            let scale = pow10(D - 18);
            StdDecimal::new(custom.0 / Uint128::from(scale))
        } else {
            // Scale up (common case)
            let scale = pow10(18 - D);
            let scaled = custom.0.checked_mul(Uint128::from(scale)).unwrap();
            StdDecimal::new(scaled)
        }
    }
}

/// Convert to Decimal256
impl<const D: u32> From<Decimal<D>> for Decimal256 {
    fn from(custom: Decimal<D>) -> Self {
        // First convert to StdDecimal, then to Decimal256
        let decimal: StdDecimal = custom.into();
        decimal.into()
    }
}

/// Try to convert from Decimal256
impl<const D: u32> TryFrom<Decimal256> for Decimal<D> {
    type Error = CustomDecimalError;

    fn try_from(value: Decimal256) -> Result<Self, Self::Error> {
        // Try to convert Decimal256 -> StdDecimal first
        let decimal =
            StdDecimal::try_from(value).map_err(|_| CustomDecimalError::ConversionError(
                "Decimal256 value too large for Decimal".to_string(),
            ))?;
        Ok(decimal.into())
    }
}

// ========== Display & FromStr ==========

impl<const D: u32> fmt::Display for Decimal<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fractional = Self::FRACTIONAL;
        let integer = self.0.u128() / fractional;
        let frac_part = self.0.u128() % fractional;

        if frac_part == 0 {
            write!(f, "{}", integer)
        } else {
            // Trim trailing zeros
            let frac_str = format!("{:0>width$}", frac_part, width = D as usize);
            let trimmed = frac_str.trim_end_matches('0');
            write!(f, "{}.{}", integer, trimmed)
        }
    }
}

impl<const D: u32> fmt::Debug for Decimal<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decimal<{}>({})", D, self)
    }
}

impl<const D: u32> FromStr for Decimal<D> {
    type Err = CustomDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        match parts.len() {
            1 => {
                // Integer only
                let integer = parts[0]
                    .parse::<u128>()
                    .map_err(|_| CustomDecimalError::ParseError(format!("Invalid integer: {}", parts[0])))?;

                Ok(Self(Uint128::from(integer * Self::FRACTIONAL)))
            }
            2 => {
                // Integer and fractional parts
                let integer = parts[0]
                    .parse::<u128>()
                    .map_err(|_| CustomDecimalError::ParseError(format!("Invalid integer: {}", parts[0])))?;

                let fractional_str = parts[1];
                if fractional_str.len() > D as usize {
                    return Err(CustomDecimalError::ParseError(format!(
                        "Too many decimal places: {} (max {})",
                        fractional_str.len(),
                        D
                    )));
                }

                let fractional = fractional_str
                    .parse::<u128>()
                    .map_err(|_| CustomDecimalError::ParseError(format!("Invalid fractional: {}", fractional_str)))?;

                // Scale to D decimals
                let scaled_fractional =
                    fractional * pow10(D - fractional_str.len() as u32);

                let total = integer
                    .checked_mul(Self::FRACTIONAL)
                    .and_then(|i| i.checked_add(scaled_fractional))
                    .ok_or(CustomDecimalError::Overflow)?;

                Ok(Self(Uint128::from(total)))
            }
            _ => Err(CustomDecimalError::ParseError(format!(
                "Invalid decimal format: {}",
                s
            ))),
        }
    }
}

// ========== Iterator Traits ==========

impl<const D: u32> Sum for Decimal<D> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + x)
    }
}

impl<'a, const D: u32> Sum<&'a Decimal<D>> for Decimal<D> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + x)
    }
}

impl<const D: u32> Product for Decimal<D> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |acc, x| acc * x)
    }
}

impl<'a, const D: u32> Product<&'a Decimal<D>> for Decimal<D> {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |acc, x| acc * x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(Decimal::<6>::ZERO.0, Uint128::zero());
        assert_eq!(Decimal::<6>::ONE.0, Uint128::new(1_000_000));
        assert_eq!(Decimal::<6>::DECIMAL_PLACES, 6);
        assert_eq!(Decimal::<6>::FRACTIONAL, 1_000_000);

        assert_eq!(Decimal::<9>::ONE.0, Uint128::new(1_000_000_000));
        assert_eq!(Decimal::<9>::DECIMAL_PLACES, 9);

        assert_eq!(Decimal::<18>::ONE.0, Uint128::new(1_000_000_000_000_000_000));
        assert_eq!(Decimal::<18>::DECIMAL_PLACES, 18);
    }

    #[test]
    fn test_raw() {
        let d = Decimal::<6>::raw(1_500_000);
        assert_eq!(d.0, Uint128::new(1_500_000));
    }

    #[test]
    fn test_from_atomics() {
        let d = Decimal::<6>::from_atomics(15u128, 1).unwrap();
        assert_eq!(d.0, Uint128::new(1_500_000)); // 1.5

        let d = Decimal::<6>::from_atomics(1_500_000u128, 6).unwrap();
        assert_eq!(d.0, Uint128::new(1_500_000)); // 1.5

        let d = Decimal::<6>::from_atomics(1_500_000_000u128, 9).unwrap();
        assert_eq!(d.0, Uint128::new(1_500_000)); // 1.5
    }

    #[test]
    fn test_percent() {
        assert_eq!(Decimal::<6>::percent(0), Decimal::<6>::ZERO);
        assert_eq!(Decimal::<6>::percent(50), Decimal::<6>::raw(500_000));
        assert_eq!(Decimal::<6>::percent(100), Decimal::<6>::ONE);
    }

    #[test]
    fn test_permille() {
        assert_eq!(Decimal::<6>::permille(0), Decimal::<6>::ZERO);
        assert_eq!(Decimal::<6>::permille(125), Decimal::<6>::raw(125_000));
        assert_eq!(Decimal::<6>::permille(1000), Decimal::<6>::ONE);
    }

    #[test]
    fn test_bps() {
        assert_eq!(Decimal::<6>::bps(0), Decimal::<6>::ZERO);
        assert_eq!(Decimal::<6>::bps(50), Decimal::<6>::raw(5_000));
        assert_eq!(Decimal::<6>::bps(10000), Decimal::<6>::ONE);
    }

    #[test]
    fn test_from_ratio() {
        let d = Decimal::<6>::from_ratio(3u128, 2u128);
        assert_eq!(d, Decimal::<6>::from_str("1.5").unwrap());

        let d = Decimal::<6>::from_ratio(1u128, 3u128);
        assert_eq!(d.0, Uint128::new(333_333)); // 0.333333
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            Decimal::<6>::from_str("1.5").unwrap(),
            Decimal::<6>::raw(1_500_000)
        );
        assert_eq!(
            Decimal::<6>::from_str("123").unwrap(),
            Decimal::<6>::raw(123_000_000)
        );
        assert_eq!(
            Decimal::<6>::from_str("0.123456").unwrap(),
            Decimal::<6>::raw(123_456)
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(Decimal::<6>::raw(1_500_000).to_string(), "1.5");
        assert_eq!(Decimal::<6>::raw(1_000_000).to_string(), "1");
        assert_eq!(Decimal::<6>::raw(123_456).to_string(), "0.123456");
        assert_eq!(Decimal::<6>::raw(100_000).to_string(), "0.1");
    }

    #[test]
    fn test_floor_ceil() {
        let d = Decimal::<6>::from_str("1.7").unwrap();
        assert_eq!(d.floor(), Decimal::<6>::from_str("1.0").unwrap());
        assert_eq!(d.ceil(), Decimal::<6>::from_str("2.0").unwrap());

        let d = Decimal::<6>::from_str("2.0").unwrap();
        assert_eq!(d.floor(), d);
        assert_eq!(d.ceil(), d);
    }

    #[test]
    fn test_sqrt() {
        let d = Decimal::<6>::from_str("4.0").unwrap();
        assert_eq!(d.sqrt(), Decimal::<6>::from_str("2.0").unwrap());

        let d = Decimal::<6>::from_str("9.0").unwrap();
        assert_eq!(d.sqrt(), Decimal::<6>::from_str("3.0").unwrap());
    }

    #[test]
    fn test_pow() {
        let d = Decimal::<6>::from_str("2.0").unwrap();
        assert_eq!(d.pow(0), Decimal::<6>::ONE);
        assert_eq!(d.pow(1), d);
        assert_eq!(d.pow(3), Decimal::<6>::from_str("8.0").unwrap());
    }

    #[test]
    fn test_decimal_conversion() {
        let custom = Decimal::<6>::from_str("1.5").unwrap();
        let decimal: StdDecimal = custom.into();
        let back: Decimal::<6> = decimal.into();
        assert_eq!(custom, back);
    }

    #[test]
    fn test_precision_conversion() {
        let d6 = Decimal::<6>::from_str("1.5").unwrap();
        let d9: Decimal<9> = d6.to_precision();
        assert_eq!(d9.atomics(), 1_500_000_000); // 1.5 with 9 decimals

        let d6_back: Decimal<6> = d9.to_precision();
        assert_eq!(d6_back, d6);
    }

    #[test]
    fn test_precision_conversion_with_precision_loss() {
        let d9 = Decimal::<9>::from_str("1.123456789").unwrap();
        let d6: Decimal<6> = d9.to_precision();
        assert_eq!(d6.to_string(), "1.123456"); // Truncated to 6 decimals
    }

    #[test]
    fn test_sum() {
        let values = vec![
            Decimal::<6>::from_str("1.0").unwrap(),
            Decimal::<6>::from_str("2.0").unwrap(),
            Decimal::<6>::from_str("3.0").unwrap(),
        ];
        let sum: Decimal<6> = values.iter().sum();
        assert_eq!(sum, Decimal::<6>::from_str("6.0").unwrap());
    }

    #[test]
    fn test_product() {
        let values = vec![
            Decimal::<6>::from_str("2.0").unwrap(),
            Decimal::<6>::from_str("3.0").unwrap(),
        ];
        let product: Decimal<6> = values.iter().product();
        assert_eq!(product, Decimal::<6>::from_str("6.0").unwrap());
    }

    #[test]
    fn test_different_precisions() {
        // Test that Decimal<9> works correctly
        let d9 = Decimal::<9>::from_str("1.123456789").unwrap();
        assert_eq!(d9.atomics(), 1_123_456_789);
        assert_eq!(d9.to_string(), "1.123456789");

        // Test Decimal<18> (same as cosmwasm_std::Decimal)
        let d18 = Decimal::<18>::from_str("1.5").unwrap();
        assert_eq!(d18.atomics(), 1_500_000_000_000_000_000);
    }

    #[test]
    fn test_type_aliases() {
        // Ensure type aliases work correctly
        let cd = CustomDecimal::from_str("1.5").unwrap();
        let d6 = Decimal6::from_str("1.5").unwrap();
        assert_eq!(cd, d6);

        let d9 = Decimal9::from_str("1.5").unwrap();
        assert_eq!(d9.atomics(), 1_500_000_000);

        let d18 = Decimal18::from_str("1.5").unwrap();
        assert_eq!(d18.atomics(), 1_500_000_000_000_000_000);
    }
}
