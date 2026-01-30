use cosmwasm_std::StdError;
use thiserror::Error;

/// Errors that can occur when working with Decimal<D>
#[derive(Error, Debug, PartialEq)]
pub enum CustomDecimalError {
    /// Overflow during arithmetic operation
    #[error("Overflow in Decimal operation")]
    Overflow,

    /// Underflow during arithmetic operation
    #[error("Underflow in Decimal operation")]
    Underflow,

    /// Attempted division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Value exceeds the valid range for Decimal
    #[error("Value exceeds valid range for Decimal")]
    RangeExceeded,

    /// Failed to parse string as Decimal
    #[error("Failed to parse Decimal: {0}")]
    ParseError(String),

    /// Conversion error between types
    #[error("Conversion error: {0}")]
    ConversionError(String),

    /// Precision conversion overflow
    #[error("Precision conversion overflow: cannot convert from {from_decimals} to {to_decimals} decimals")]
    PrecisionConversionOverflow {
        from_decimals: u32,
        to_decimals: u32,
    },
}

/// Convert CustomDecimalError to CosmWasm's StdError
impl From<CustomDecimalError> for StdError {
    fn from(err: CustomDecimalError) -> Self {
        match err {
            CustomDecimalError::Overflow => StdError::generic_err("Decimal overflow"),
            CustomDecimalError::Underflow => StdError::generic_err("Decimal underflow"),
            CustomDecimalError::DivisionByZero => StdError::generic_err("Division by zero"),
            CustomDecimalError::RangeExceeded => {
                StdError::generic_err("Value exceeds valid range")
            }
            CustomDecimalError::ParseError(msg) => {
                StdError::generic_err(format!("Parse error: {}", msg))
            }
            CustomDecimalError::ConversionError(msg) => {
                StdError::generic_err(format!("Conversion error: {}", msg))
            }
            CustomDecimalError::PrecisionConversionOverflow { from_decimals, to_decimals } => {
                StdError::generic_err(format!(
                    "Precision conversion overflow: cannot convert from {} to {} decimals",
                    from_decimals, to_decimals
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion_to_std_error() {
        let err = CustomDecimalError::Overflow;
        let std_err: StdError = err.into();
        assert!(std_err.to_string().contains("overflow"));

        let err = CustomDecimalError::DivisionByZero;
        let std_err: StdError = err.into();
        assert!(std_err.to_string().contains("Division by zero"));

        let err = CustomDecimalError::ParseError("invalid format".to_string());
        let std_err: StdError = err.into();
        assert!(std_err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            CustomDecimalError::Overflow.to_string(),
            "Overflow in Decimal operation"
        );
        assert_eq!(
            CustomDecimalError::DivisionByZero.to_string(),
            "Division by zero"
        );
        assert_eq!(
            CustomDecimalError::ParseError("test".to_string()).to_string(),
            "Failed to parse Decimal: test"
        );
    }

    #[test]
    fn test_precision_conversion_error() {
        let err = CustomDecimalError::PrecisionConversionOverflow {
            from_decimals: 6,
            to_decimals: 18,
        };
        assert_eq!(
            err.to_string(),
            "Precision conversion overflow: cannot convert from 6 to 18 decimals"
        );

        let std_err: StdError = err.into();
        assert!(std_err.to_string().contains("6 to 18"));
    }
}
