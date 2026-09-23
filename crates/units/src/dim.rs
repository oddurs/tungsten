//! Dimensions as vectors of rational exponents over the base dimensions.

use crate::Rational;
use std::fmt;

/// The base dimensions, in vector order. See docs/concept.md §3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseDim {
    Length,
    Mass,
    Time,
    Current,
    Temperature,
    Amount,
    Luminosity,
    Currency,
    Information,
    Count,
}

impl BaseDim {
    pub const ALL: [Self; 10] = [
        Self::Length,
        Self::Mass,
        Self::Time,
        Self::Current,
        Self::Temperature,
        Self::Amount,
        Self::Luminosity,
        Self::Currency,
        Self::Information,
        Self::Count,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Mass => "mass",
            Self::Time => "time",
            Self::Current => "electric current",
            Self::Temperature => "temperature",
            Self::Amount => "amount of substance",
            Self::Luminosity => "luminous intensity",
            Self::Currency => "money",
            Self::Information => "information",
            Self::Count => "count",
        }
    }

    /// Short name used when composing unnamed dimensions: `length²·time⁻¹`.
    pub const fn short(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Amount => "amount",
            Self::Luminosity => "luminosity",
            other => other.name(),
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|b| b.name() == s || b.short() == s)
    }
}

/// A dimension: one rational exponent per [`BaseDim`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dim(pub [Rational; 10]);

impl Dim {
    pub const NONE: Self = Self([Rational::ZERO; 10]);

    pub fn base(b: BaseDim) -> Self {
        let mut d = Self::NONE;
        d.0[b as usize] = Rational::ONE;
        d
    }

    pub fn is_none(&self) -> bool {
        self.0.iter().all(|e| e.is_zero())
    }

    pub fn exponent(&self, b: BaseDim) -> Rational {
        self.0[b as usize]
    }

    /// The single base dimension this is, to the first power, if any.
    pub fn as_base(&self) -> Option<BaseDim> {
        let mut found = None;
        for (i, e) in self.0.iter().enumerate() {
            if e.is_zero() {
                continue;
            }
            if *e != Rational::ONE || found.is_some() {
                return None;
            }
            found = Some(BaseDim::ALL[i]);
        }
        found
    }

    fn zip(&self, o: &Self, f: impl Fn(Rational, Rational) -> Option<Rational>) -> Option<Self> {
        let mut out = Self::NONE;
        for i in 0..10 {
            out.0[i] = f(self.0[i], o.0[i])?;
        }
        Some(out)
    }

    /// Product of two quantities' dimensions. `None` only on absurd exponents.
    pub fn mul(&self, o: &Self) -> Option<Self> {
        self.zip(o, Rational::checked_add)
    }

    pub fn div(&self, o: &Self) -> Option<Self> {
        self.zip(o, Rational::checked_sub)
    }

    pub fn pow(&self, e: Rational) -> Option<Self> {
        let mut out = Self::NONE;
        for i in 0..10 {
            out.0[i] = self.0[i].checked_mul(e)?;
        }
        Some(out)
    }

    pub fn recip(&self) -> Self {
        let mut out = Self::NONE;
        for i in 0..10 {
            out.0[i] = self.0[i].checked_neg().unwrap_or(self.0[i]);
        }
        out
    }

    /// Composes a description from base names: `length²·time⁻¹`.
    pub fn compose(&self, superscripts: bool) -> String {
        let mut parts = Vec::new();
        for (i, e) in self.0.iter().enumerate() {
            if e.is_zero() {
                continue;
            }
            let name = BaseDim::ALL[i].short();
            if *e == Rational::ONE {
                parts.push(name.to_string());
            } else {
                parts.push(format!("{name}{}", exponent(*e, superscripts)));
            }
        }
        if parts.is_empty() {
            "dimensionless".into()
        } else {
            parts.join(if superscripts { "·" } else { "*" })
        }
    }
}

/// Formats an exponent: `²`, `⁻¹`, `^(1/2)`; or `^2`, `^-1` when plain.
pub fn exponent(e: Rational, superscripts: bool) -> String {
    if !e.is_integer() {
        return format!("^({e})");
    }
    if !superscripts {
        return format!("^{e}");
    }
    e.num()
        .to_string()
        .chars()
        .map(|c| match c {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            _ => '⁹',
        })
        .collect()
}

impl fmt::Debug for Dim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.compose(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebra() {
        let l = Dim::base(BaseDim::Length);
        let t = Dim::base(BaseDim::Time);
        let v = l.div(&t).unwrap();
        assert_eq!(v.compose(true), "length·time⁻¹");
        let area = l.mul(&l).unwrap();
        assert_eq!(area.pow(Rational::new(1, 2).unwrap()).unwrap(), l);
        assert_eq!(
            l.pow(Rational::new(1, 2).unwrap()).unwrap().compose(true),
            "length^(1/2)"
        );
        assert_eq!(v.mul(&t).unwrap().as_base(), Some(BaseDim::Length));
        assert!(l.div(&l).unwrap().is_none());
    }
}
