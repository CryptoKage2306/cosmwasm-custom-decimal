use crate::Decimal;
use cosmwasm_std::{Uint128, Uint256};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

// ========== Macros to reduce boilerplate ==========

/// Macro to implement binary operations for all 4 combinations:
/// - Decimal<D> op Decimal<D>
/// - Decimal<D> op &Decimal<D>
/// - &Decimal<D> op Decimal<D>
/// - &Decimal<D> op &Decimal<D>
macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $impl_fn:ident) => {
        // Decimal<D> op Decimal<D>
        impl<const D: u32> $trait for Decimal<D> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                $impl_fn(self, rhs)
            }
        }

        // &Decimal<D> op Decimal<D>
        impl<const D: u32> $trait<Decimal<D>> for &Decimal<D> {
            type Output = Decimal<D>;

            fn $method(self, rhs: Decimal<D>) -> Self::Output {
                $impl_fn(*self, rhs)
            }
        }

        // Decimal<D> op &Decimal<D>
        impl<const D: u32> $trait<&Decimal<D>> for Decimal<D> {
            type Output = Decimal<D>;

            fn $method(self, rhs: &Decimal<D>) -> Self::Output {
                $impl_fn(self, *rhs)
            }
        }

        // &Decimal<D> op &Decimal<D>
        impl<const D: u32> $trait<&Decimal<D>> for &Decimal<D> {
            type Output = Decimal<D>;

            fn $method(self, rhs: &Decimal<D>) -> Self::Output {
                $impl_fn(*self, *rhs)
            }
        }
    };
}

/// Macro to implement assignment operations for both owned and borrowed rhs
macro_rules! impl_assign_op {
    ($trait:ident, $method:ident, $op:tt) => {
        // Decimal<D> op= Decimal<D>
        impl<const D: u32> $trait for Decimal<D> {
            fn $method(&mut self, rhs: Self) {
                *self = *self $op rhs;
            }
        }

        // Decimal<D> op= &Decimal<D>
        impl<const D: u32> $trait<&Decimal<D>> for Decimal<D> {
            fn $method(&mut self, rhs: &Decimal<D>) {
                *self = *self $op rhs;
            }
        }
    };
}

// ========== Addition ==========

fn add_impl<const D: u32>(a: Decimal<D>, b: Decimal<D>) -> Decimal<D> {
    Decimal(a.0.checked_add(b.0).expect("attempt to add with overflow"))
}

impl_binary_op!(Add, add, add_impl);
impl_assign_op!(AddAssign, add_assign, +);

// ========== Subtraction ==========

fn sub_impl<const D: u32>(a: Decimal<D>, b: Decimal<D>) -> Decimal<D> {
    Decimal(
        a.0.checked_sub(b.0)
            .expect("attempt to subtract with overflow"),
    )
}

impl_binary_op!(Sub, sub, sub_impl);
impl_assign_op!(SubAssign, sub_assign, -);

// ========== Multiplication ==========

fn mul_impl<const D: u32>(a: Decimal<D>, b: Decimal<D>) -> Decimal<D> {
    // Use Uint256 to prevent overflow
    let result = Uint256::from(a.0)
        .checked_mul(Uint256::from(b.0))
        .unwrap()
        .checked_div(Uint256::from(Decimal::<D>::FRACTIONAL))
        .unwrap();

    Decimal(
        Uint128::try_from(result).expect("multiplication result exceeds Uint128 range"),
    )
}

impl_binary_op!(Mul, mul, mul_impl);
impl_assign_op!(MulAssign, mul_assign, *);

// ========== Division ==========

fn div_impl<const D: u32>(a: Decimal<D>, b: Decimal<D>) -> Decimal<D> {
    if b.0.is_zero() {
        panic!("Division by zero");
    }

    // Use Uint256 to prevent overflow
    let numerator = Uint256::from(a.0).checked_mul(Uint256::from(Decimal::<D>::FRACTIONAL))
        .unwrap();
    let result = numerator
        .checked_div(Uint256::from(b.0))
        .unwrap();

    Decimal(Uint128::try_from(result).expect("division result exceeds Uint128 range"))
}

impl_binary_op!(Div, div, div_impl);
impl_assign_op!(DivAssign, div_assign, /);

// ========== Remainder ==========

fn rem_impl<const D: u32>(a: Decimal<D>, b: Decimal<D>) -> Decimal<D> {
    if b.0.is_zero() {
        panic!("Division by zero");
    }
    Decimal(a.0.checked_rem(b.0).unwrap())
}

impl_binary_op!(Rem, rem, rem_impl);
impl_assign_op!(RemAssign, rem_assign, %);

// ========== Negation ==========

impl<const D: u32> Neg for Decimal<D> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.0.is_zero() {
            self
        } else {
            panic!("Negation of non-zero Decimal is not supported")
        }
    }
}

impl<const D: u32> Neg for &Decimal<D> {
    type Output = Decimal<D>;

    fn neg(self) -> Self::Output {
        if self.0.is_zero() {
            *self
        } else {
            panic!("Negation of non-zero Decimal is not supported")
        }
    }
}

// ========== Operations with Uint128 ==========

// Decimal<D> * Uint128 -> Uint128
impl<const D: u32> Mul<Uint128> for Decimal<D> {
    type Output = Uint128;

    fn mul(self, rhs: Uint128) -> Self::Output {
        // Use Uint256 to prevent overflow
        let result = Uint256::from(self.0)
            .checked_mul(Uint256::from(rhs))
            .unwrap()
            .checked_div(Uint256::from(Self::FRACTIONAL))
            .unwrap();

        Uint128::try_from(result).expect("multiplication result exceeds Uint128 range")
    }
}

// &Decimal<D> * Uint128 -> Uint128
impl<const D: u32> Mul<Uint128> for &Decimal<D> {
    type Output = Uint128;

    fn mul(self, rhs: Uint128) -> Self::Output {
        *self * rhs
    }
}

// Decimal<D> * &Uint128 -> Uint128
impl<const D: u32> Mul<&Uint128> for Decimal<D> {
    type Output = Uint128;

    fn mul(self, rhs: &Uint128) -> Self::Output {
        self * *rhs
    }
}

// &Decimal<D> * &Uint128 -> Uint128
impl<const D: u32> Mul<&Uint128> for &Decimal<D> {
    type Output = Uint128;

    fn mul(self, rhs: &Uint128) -> Self::Output {
        *self * *rhs
    }
}

// Uint128 * Decimal<D> -> Uint128 (commutative)
impl<const D: u32> Mul<Decimal<D>> for Uint128 {
    type Output = Uint128;

    fn mul(self, rhs: Decimal<D>) -> Self::Output {
        rhs * self
    }
}

// &Uint128 * Decimal<D> -> Uint128
impl<const D: u32> Mul<Decimal<D>> for &Uint128 {
    type Output = Uint128;

    fn mul(self, rhs: Decimal<D>) -> Self::Output {
        rhs * *self
    }
}

// Uint128 * &Decimal<D> -> Uint128
impl<const D: u32> Mul<&Decimal<D>> for Uint128 {
    type Output = Uint128;

    fn mul(self, rhs: &Decimal<D>) -> Self::Output {
        *rhs * self
    }
}

// &Uint128 * &Decimal<D> -> Uint128
impl<const D: u32> Mul<&Decimal<D>> for &Uint128 {
    type Output = Uint128;

    fn mul(self, rhs: &Decimal<D>) -> Self::Output {
        *rhs * *self
    }
}

// Decimal<D> / Uint128 -> Decimal<D>
impl<const D: u32> Div<Uint128> for Decimal<D> {
    type Output = Decimal<D>;

    fn div(self, rhs: Uint128) -> Self::Output {
        if rhs.is_zero() {
            panic!("Division by zero");
        }
        Decimal(self.0.checked_div(rhs).unwrap())
    }
}

// &Decimal<D> / Uint128 -> Decimal<D>
impl<const D: u32> Div<Uint128> for &Decimal<D> {
    type Output = Decimal<D>;

    fn div(self, rhs: Uint128) -> Self::Output {
        *self / rhs
    }
}

// Decimal<D> / &Uint128 -> Decimal<D>
impl<const D: u32> Div<&Uint128> for Decimal<D> {
    type Output = Decimal<D>;

    fn div(self, rhs: &Uint128) -> Self::Output {
        self / *rhs
    }
}

// &Decimal<D> / &Uint128 -> Decimal<D>
impl<const D: u32> Div<&Uint128> for &Decimal<D> {
    type Output = Decimal<D>;

    fn div(self, rhs: &Uint128) -> Self::Output {
        *self / *rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decimal6, Decimal9};

    #[test]
    fn test_addition() {
        let a = Decimal::<6>(Uint128::new(1_500_000)); // 1.5
        let b = Decimal::<6>(Uint128::new(2_500_000)); // 2.5
        let result = a + b;
        assert_eq!(result.0, Uint128::new(4_000_000)); // 4.0
    }

    #[test]
    fn test_subtraction() {
        let a = Decimal::<6>(Uint128::new(5_000_000)); // 5.0
        let b = Decimal::<6>(Uint128::new(2_000_000)); // 2.0
        let result = a - b;
        assert_eq!(result.0, Uint128::new(3_000_000)); // 3.0
    }

    #[test]
    fn test_multiplication() {
        let a = Decimal::<6>(Uint128::new(2_000_000)); // 2.0
        let b = Decimal::<6>(Uint128::new(3_000_000)); // 3.0
        let result = a * b;
        assert_eq!(result.0, Uint128::new(6_000_000)); // 6.0
    }

    #[test]
    fn test_division() {
        let a = Decimal::<6>(Uint128::new(6_000_000)); // 6.0
        let b = Decimal::<6>(Uint128::new(2_000_000)); // 2.0
        let result = a / b;
        assert_eq!(result.0, Uint128::new(3_000_000)); // 3.0
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero() {
        let a = Decimal::<6>(Uint128::new(1_000_000));
        let b = Decimal::<6>(Uint128::zero());
        let _ = a / b;
    }

    #[test]
    fn test_mul_uint128() {
        let decimal = Decimal::<6>(Uint128::new(2_500_000)); // 2.5
        let amount = Uint128::new(1000);
        let result = decimal * amount;
        assert_eq!(result, Uint128::new(2500)); // 2.5 * 1000 = 2500
    }

    #[test]
    fn test_uint128_mul_decimal() {
        let amount = Uint128::new(1000);
        let decimal = Decimal::<6>(Uint128::new(2_500_000)); // 2.5
        let result = amount * decimal;
        assert_eq!(result, Uint128::new(2500)); // 1000 * 2.5 = 2500
    }

    #[test]
    fn test_div_uint128() {
        let decimal = Decimal::<6>(Uint128::new(10_000_000)); // 10.0
        let divisor = Uint128::new(2);
        let result = decimal / divisor;
        assert_eq!(result.0, Uint128::new(5_000_000)); // 5.0
    }

    #[test]
    fn test_reference_operations() {
        let a = Decimal::<6>(Uint128::new(1_000_000));
        let b = Decimal::<6>(Uint128::new(2_000_000));

        // Test all reference combinations
        assert_eq!(&a + &b, Decimal::<6>(Uint128::new(3_000_000)));
        assert_eq!(a + &b, Decimal::<6>(Uint128::new(3_000_000)));
        assert_eq!(&a + b, Decimal::<6>(Uint128::new(3_000_000)));
        assert_eq!(a + b, Decimal::<6>(Uint128::new(3_000_000)));
    }

    #[test]
    fn test_assign_operations() {
        let mut a = Decimal::<6>(Uint128::new(1_000_000));
        a += Decimal::<6>(Uint128::new(2_000_000));
        assert_eq!(a.0, Uint128::new(3_000_000));

        a -= Decimal::<6>(Uint128::new(1_000_000));
        assert_eq!(a.0, Uint128::new(2_000_000));

        a *= Decimal::<6>(Uint128::new(2_000_000));
        assert_eq!(a.0, Uint128::new(4_000_000));

        a /= Decimal::<6>(Uint128::new(2_000_000));
        assert_eq!(a.0, Uint128::new(2_000_000));
    }

    #[test]
    fn test_negation_of_zero() {
        let zero = Decimal::<6>(Uint128::zero());
        let neg_zero = -zero;
        assert_eq!(neg_zero, zero);
    }

    #[test]
    #[should_panic(expected = "Negation of non-zero Decimal is not supported")]
    fn test_negation_of_non_zero() {
        let non_zero = Decimal::<6>(Uint128::new(1_000_000));
        let _ = -non_zero;
    }

    #[test]
    fn test_decimal9_operations() {
        let a = Decimal9::raw(1_500_000_000); // 1.5
        let b = Decimal9::raw(2_500_000_000); // 2.5
        let result = a + b;
        assert_eq!(result.atomics(), 4_000_000_000); // 4.0

        let product = a * b;
        assert_eq!(product.atomics(), 3_750_000_000); // 3.75
    }

    #[test]
    fn test_type_aliases() {
        let d6 = Decimal6::raw(1_000_000);
        let d9 = Decimal9::raw(1_000_000_000);

        assert_eq!(d6.atomics(), 1_000_000);
        assert_eq!(d9.atomics(), 1_000_000_000);
    }
}
