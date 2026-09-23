//! Exact rationals over `i128`, with checked arithmetic.
//!
//! Every operation returns `None` on overflow rather than wrapping, so callers
//! can fall back to floating point instead of producing a wrong exact answer.

use std::cmp::Ordering;
use std::fmt;

/// A reduced fraction with a positive denominator.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
    num: i128,
    den: i128,
}

impl Default for Rational {
    fn default() -> Self {
        Self::ZERO
    }
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Rational {
    pub const ZERO: Self = Self { num: 0, den: 1 };
    pub const ONE: Self = Self { num: 1, den: 1 };

    /// `num / den`, reduced. `None` when `den` is zero or reduction overflows.
    pub fn new(num: i128, den: i128) -> Option<Self> {
        if den == 0 {
            return None;
        }
        let g = gcd(num, den).max(1);
        let (mut n, mut d) = (num / g, den / g);
        if d < 0 {
            n = n.checked_neg()?;
            d = d.checked_neg()?;
        }
        Some(Self { num: n, den: d })
    }

    pub const fn int(n: i128) -> Self {
        Self { num: n, den: 1 }
    }

    pub const fn num(self) -> i128 {
        self.num
    }

    pub const fn den(self) -> i128 {
        self.den
    }

    pub const fn is_zero(self) -> bool {
        self.num == 0
    }

    pub const fn is_integer(self) -> bool {
        self.den == 1
    }

    pub const fn is_negative(self) -> bool {
        self.num < 0
    }

    pub fn abs(self) -> Self {
        Self {
            num: self.num.abs(),
            den: self.den,
        }
    }

    pub fn checked_add(self, o: Self) -> Option<Self> {
        let g = gcd(self.den, o.den);
        let l = self.den / g;
        let r = o.den / g;
        let num = self
            .num
            .checked_mul(r)?
            .checked_add(o.num.checked_mul(l)?)?;
        let den = self.den.checked_mul(r)?;
        Self::new(num, den)
    }

    pub fn checked_sub(self, o: Self) -> Option<Self> {
        self.checked_add(o.checked_neg()?)
    }

    pub fn checked_neg(self) -> Option<Self> {
        Some(Self {
            num: self.num.checked_neg()?,
            den: self.den,
        })
    }

    pub fn checked_mul(self, o: Self) -> Option<Self> {
        // Cross-reduce first so products stay small.
        let g1 = gcd(self.num, o.den).max(1);
        let g2 = gcd(o.num, self.den).max(1);
        let num = (self.num / g1).checked_mul(o.num / g2)?;
        let den = (self.den / g2).checked_mul(o.den / g1)?;
        Self::new(num, den)
    }

    pub fn checked_div(self, o: Self) -> Option<Self> {
        self.checked_mul(o.checked_recip()?)
    }

    pub fn checked_recip(self) -> Option<Self> {
        if self.num == 0 {
            return None;
        }
        Self::new(self.den, self.num)
    }

    /// Integer power. Negative exponents invert.
    pub fn checked_pow(self, exp: i128) -> Option<Self> {
        if exp < 0 {
            return self.checked_recip()?.checked_pow(exp.checked_neg()?);
        }
        let e = u32::try_from(exp).ok()?;
        Some(Self {
            num: self.num.checked_pow(e)?,
            den: self.den.checked_pow(e)?,
        })
    }

    /// The exact `n`th root, when one exists.
    pub fn exact_root(self, n: i128) -> Option<Self> {
        if n <= 0 {
            return None;
        }
        if n == 1 {
            return Some(self);
        }
        let neg = self.num < 0;
        if neg && n % 2 == 0 {
            return None;
        }
        let num = int_root(self.num.unsigned_abs(), n)?;
        let den = int_root(self.den.unsigned_abs(), n)?;
        let num = i128::try_from(num).ok()?;
        let den = i128::try_from(den).ok()?;
        Self::new(if neg { -num } else { num }, den)
    }

    pub fn floor(self) -> i128 {
        self.num.div_euclid(self.den)
    }

    pub fn to_f64(self) -> f64 {
        // Divide in f64 but keep precision for large parts where possible.
        self.num as f64 / self.den as f64
    }

    /// True when the decimal expansion terminates (denominator is 2^a·5^b).
    pub fn is_terminating(self) -> bool {
        let mut d = self.den;
        while d % 2 == 0 {
            d /= 2;
        }
        while d % 5 == 0 {
            d /= 5;
        }
        d == 1
    }

    /// Parses `12`, `-1.5`, `2.5e-3`, `1e30` and `3/4` exactly.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if let Some((a, b)) = s.split_once('/') {
            return Self::parse(a)?.checked_div(Self::parse(b)?);
        }
        let (mantissa, exp) = match s.find(['e', 'E']) {
            Some(i) => (&s[..i], s[i + 1..].parse::<i32>().ok()?),
            None => (s, 0),
        };
        let (neg, mantissa) = match mantissa.strip_prefix('-') {
            Some(m) => (true, m),
            None => (false, mantissa.strip_prefix('+').unwrap_or(mantissa)),
        };
        let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        if int_part.is_empty() && frac_part.is_empty() {
            return None;
        }
        if !int_part
            .bytes()
            .chain(frac_part.bytes())
            .all(|b| b.is_ascii_digit())
        {
            return None;
        }
        let digits = format!("{int_part}{frac_part}");
        let mut num: i128 = if digits.is_empty() {
            0
        } else {
            digits.parse().ok()?
        };
        if neg {
            num = -num;
        }
        let scale = exp - i32::try_from(frac_part.len()).ok()?;
        let ten = Self::int(10);
        let r = Self::int(num);
        if scale >= 0 {
            r.checked_mul(ten.checked_pow(i128::from(scale))?)
        } else {
            r.checked_div(ten.checked_pow(i128::from(-scale))?)
        }
    }
}

/// Integer `n`th root of `x`, if `x` is a perfect power.
fn int_root(x: u128, n: i128) -> Option<u128> {
    if x < 2 {
        return Some(x);
    }
    let n32 = u32::try_from(n).ok()?;
    let guess = (x as f64).powf(1.0 / n as f64).round() as u128;
    (guess.saturating_sub(1)..=guess + 1).find(|c| c.checked_pow(n32) == Some(x))
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        match (
            self.num.checked_mul(other.den),
            other.num.checked_mul(self.den),
        ) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => self.to_f64().total_cmp(&other.to_f64()),
        }
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl fmt::Debug for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(s: &str) -> Rational {
        Rational::parse(s).unwrap()
    }

    #[test]
    fn parses_decimals_exactly() {
        assert_eq!(q("0.3048"), Rational::new(381, 1250).unwrap());
        assert_eq!(q("1.5e3"), Rational::int(1500));
        assert_eq!(q("2.5e-3"), Rational::new(1, 400).unwrap());
        assert_eq!(q("-3/4"), Rational::new(-3, 4).unwrap());
        assert_eq!(q(".5"), Rational::new(1, 2).unwrap());
        assert!(Rational::parse("1.2.3").is_none());
    }

    #[test]
    fn arithmetic() {
        assert_eq!(q("1/3").checked_add(q("1/6")), Some(q("1/2")));
        assert_eq!(q("2/3").checked_pow(-2), Some(q("9/4")));
        assert_eq!(q("9/4").exact_root(2), Some(q("3/2")));
        assert_eq!(q("-8").exact_root(3), Some(q("-2")));
        assert_eq!(q("2").exact_root(2), None);
        assert_eq!(q("-7/2").floor(), -4);
    }

    #[test]
    fn overflow_is_none() {
        let big = Rational::int(i128::MAX / 2);
        assert!(big.checked_mul(Rational::int(4)).is_none());
        assert!(Rational::int(10).checked_pow(60).is_none());
    }

    #[test]
    fn terminating() {
        assert!(q("1/8").is_terminating());
        assert!(!q("1/3").is_terminating());
    }
}
