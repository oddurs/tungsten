//! Numbers that stay exact for as long as they can.
//!
//! A [`Number`] is an exact rational until an operation has no exact answer
//! (`sqrt(2)`, `sin`, overflow), after which it is an `f64` and remembers that
//! exactness was lost.
//!
//! Exact numbers also carry a *decimal* flavour. Integers and fractions typed
//! by the user are not decimal, so `1/3 + 1/6` can be shown as `1/2`. Decimal
//! literals and every unit conversion factor are decimal, so `1.5 * 3` shows as
//! `4.5` and `100 °F in °C` as `37.78`, never as a fraction.

use crate::Rational;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum Number {
    Exact { value: Rational, decimal: bool },
    Approx(f64),
}

/// Why an arithmetic operation has no answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathError {
    DivideByZero,
    /// Even root of a negative number, log of a non-positive number, and so on.
    NotReal,
    /// The result is too large to represent.
    Overflow,
}

// add/sub/mul/div return Result so overflow and division by zero cannot be
// ignored, which is why they are not the std::ops traits.
#[allow(clippy::should_implement_trait)]
impl Number {
    pub const ZERO: Self = Self::Exact {
        value: Rational::ZERO,
        decimal: false,
    };
    pub const ONE: Self = Self::Exact {
        value: Rational::ONE,
        decimal: false,
    };

    pub const fn int(n: i128) -> Self {
        Self::Exact {
            value: Rational::int(n),
            decimal: false,
        }
    }

    pub const fn exact(value: Rational, decimal: bool) -> Self {
        Self::Exact { value, decimal }
    }

    pub fn approx(x: f64) -> Result<Self, MathError> {
        if x.is_nan() {
            Err(MathError::NotReal)
        } else if x.is_infinite() {
            Err(MathError::Overflow)
        } else {
            Ok(Self::Approx(x))
        }
    }

    pub fn to_f64(self) -> f64 {
        match self {
            Self::Exact { value, .. } => value.to_f64(),
            Self::Approx(x) => x,
        }
    }

    pub fn as_rational(self) -> Option<Rational> {
        match self {
            Self::Exact { value, .. } => Some(value),
            Self::Approx(_) => None,
        }
    }

    pub fn is_exact(self) -> bool {
        matches!(self, Self::Exact { .. })
    }

    pub fn is_decimal(self) -> bool {
        match self {
            Self::Exact { decimal, .. } => decimal,
            Self::Approx(_) => true,
        }
    }

    pub fn with_decimal(self, decimal: bool) -> Self {
        match self {
            Self::Exact { value, .. } => Self::Exact { value, decimal },
            other => other,
        }
    }

    pub fn is_zero(self) -> bool {
        match self {
            Self::Exact { value, .. } => value.is_zero(),
            Self::Approx(x) => x == 0.0,
        }
    }

    pub fn is_negative(self) -> bool {
        match self {
            Self::Exact { value, .. } => value.is_negative(),
            Self::Approx(x) => x < 0.0,
        }
    }

    pub fn is_integer(self) -> bool {
        match self {
            Self::Exact { value, .. } => value.is_integer(),
            Self::Approx(x) => x.fract() == 0.0,
        }
    }

    fn binary(
        self,
        o: Self,
        exact: impl Fn(Rational, Rational) -> Option<Rational>,
        float: impl Fn(f64, f64) -> f64,
    ) -> Result<Self, MathError> {
        if let (
            Self::Exact {
                value: a,
                decimal: da,
            },
            Self::Exact {
                value: b,
                decimal: db,
            },
        ) = (self, o)
            && let Some(r) = exact(a, b)
        {
            return Ok(Self::Exact {
                value: r,
                decimal: da || db,
            });
        }
        Self::approx(float(self.to_f64(), o.to_f64()))
    }

    pub fn add(self, o: Self) -> Result<Self, MathError> {
        self.binary(o, Rational::checked_add, |a, b| a + b)
    }

    pub fn sub(self, o: Self) -> Result<Self, MathError> {
        self.binary(o, Rational::checked_sub, |a, b| a - b)
    }

    pub fn mul(self, o: Self) -> Result<Self, MathError> {
        self.binary(o, Rational::checked_mul, |a, b| a * b)
    }

    pub fn div(self, o: Self) -> Result<Self, MathError> {
        if o.is_zero() {
            return Err(MathError::DivideByZero);
        }
        self.binary(o, Rational::checked_div, |a, b| a / b)
    }

    pub fn neg(self) -> Self {
        match self {
            Self::Exact { value, decimal } => match value.checked_neg() {
                Some(v) => Self::Exact { value: v, decimal },
                None => Self::Approx(-value.to_f64()),
            },
            Self::Approx(x) => Self::Approx(-x),
        }
    }

    pub fn abs(self) -> Self {
        if self.is_negative() { self.neg() } else { self }
    }

    /// `self ^ exp`, exact whenever the answer is rational.
    pub fn pow(self, exp: Self) -> Result<Self, MathError> {
        if let (
            Self::Exact { value: b, decimal },
            Self::Exact {
                value: e,
                decimal: de,
            },
        ) = (self, exp)
        {
            if b.is_zero() && e.is_negative() {
                return Err(MathError::DivideByZero);
            }
            let decimal = decimal || de;
            if e.is_integer() {
                if let Some(r) = b.checked_pow(e.num()) {
                    return Ok(Self::Exact { value: r, decimal });
                }
            } else if let Some(root) = b.exact_root(e.den())
                && let Some(r) = root.checked_pow(e.num())
            {
                return Ok(Self::Exact { value: r, decimal });
            }
            if b.is_negative() && !e.is_integer() {
                // A real odd root of a negative number: -(|b|^e).
                if e.den() % 2 == 1 {
                    let mag = (-b.to_f64()).powf(e.to_f64());
                    let signed = if e.num() % 2 == 0 { mag } else { -mag };
                    return Self::approx(signed);
                }
                return Err(MathError::NotReal);
            }
        }
        let (b, e) = (self.to_f64(), exp.to_f64());
        if b < 0.0 && e.fract() != 0.0 {
            return Err(MathError::NotReal);
        }
        Self::approx(b.powf(e))
    }

    /// Applies a float function, keeping exactness only through `exact`.
    pub fn map(
        self,
        exact: impl Fn(Rational) -> Option<Rational>,
        float: impl Fn(f64) -> f64,
    ) -> Result<Self, MathError> {
        if let Self::Exact { value, decimal } = self
            && let Some(r) = exact(value)
        {
            return Ok(Self::Exact { value: r, decimal });
        }
        Self::approx(float(self.to_f64()))
    }

    pub fn partial_cmp(self, o: Self) -> Option<std::cmp::Ordering> {
        match (self, o) {
            (Self::Exact { value: a, .. }, Self::Exact { value: b, .. }) => Some(a.cmp(&b)),
            _ => self.to_f64().partial_cmp(&o.to_f64()),
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact {
                value,
                decimal: true,
            } => write!(f, "{value}d"),
            Self::Exact {
                value,
                decimal: false,
            } => write!(f, "{value}"),
            Self::Approx(x) => write!(f, "~{x}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(s: &str) -> Number {
        Number::exact(Rational::parse(s).unwrap(), false)
    }

    #[test]
    fn exact_until_irrational() {
        assert_eq!(q("9").pow(q("1/2")), Ok(q("3")));
        assert!(!q("2").pow(q("1/2")).unwrap().is_exact());
        assert_eq!(q("-8").pow(q("1/3")).unwrap().to_f64(), -2.0);
        assert_eq!(q("-4").pow(q("1/2")), Err(MathError::NotReal));
    }

    #[test]
    fn overflow_falls_back_to_float() {
        let big = q("10").pow(q("30")).unwrap();
        assert!(big.is_exact());
        let bigger = big.mul(big).unwrap();
        assert!(!bigger.is_exact());
        assert!((bigger.to_f64() - 1e60).abs() / 1e60 < 1e-12);
    }

    #[test]
    fn decimal_flavour_propagates() {
        let d = Number::exact(Rational::parse("1.5").unwrap(), true);
        assert!(d.mul(q("3")).unwrap().is_decimal());
        assert!(!q("1/3").add(q("1/6")).unwrap().is_decimal());
    }

    #[test]
    fn divide_by_zero() {
        assert_eq!(q("1").div(q("0")), Err(MathError::DivideByZero));
    }
}
