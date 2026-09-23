//! Unit expressions: `km/h`, `kg·m/s²`, `m^(1/2)`.
//!
//! [`UnitExpr`] is how a quantity remembers the units it should be shown in.
//! Values themselves are always kept in SI base units; the expression only
//! decides display, which is why simplifying it never changes a number.

use crate::table::{self, UnitRef};
use crate::{Dim, Number, Rational, dim::exponent, quantity};

/// A product of units raised to rational powers, in first-seen order.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct UnitExpr(Vec<(UnitRef, Rational)>);

impl UnitExpr {
    pub fn one(u: UnitRef) -> Self {
        Self(vec![(u, Rational::ONE)])
    }

    /// Builds from terms, merging repeats and dropping zero exponents.
    pub fn from_terms(terms: impl IntoIterator<Item = (UnitRef, Rational)>) -> Self {
        let mut out: Vec<(UnitRef, Rational)> = Vec::new();
        for (u, e) in terms {
            match out.iter_mut().find(|(v, _)| *v == u) {
                Some((_, acc)) => *acc = acc.checked_add(e).unwrap_or(*acc),
                None => out.push((u, e)),
            }
        }
        out.retain(|(_, e)| !e.is_zero());
        Self(out)
    }

    /// Parses a unit expression such as `km/h` or `kg m^2/s^2`. No numbers
    /// other than a leading `1` (`1/d`) are allowed.
    pub fn parse(s: &str) -> Option<Self> {
        let p = parse_def(s).ok()?;
        (p.coef.as_rational() == Some(Rational::ONE)).then(|| Self::from_terms(p.terms))
    }

    pub fn terms(&self) -> &[(UnitRef, Rational)] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The single unit this is, to the first power.
    pub fn single(&self) -> Option<UnitRef> {
        match self.0.as_slice() {
            [(u, e)] if *e == Rational::ONE => Some(*u),
            _ => None,
        }
    }

    /// Size in SI base units.
    pub fn factor(&self) -> Number {
        let mut f = Number::ONE;
        for (u, e) in &self.0 {
            let t = u
                .factor()
                .pow(Number::exact(*e, false))
                .unwrap_or(Number::ONE);
            f = f.mul(t).unwrap_or(f);
        }
        f
    }

    pub fn dim(&self) -> Dim {
        self.0.iter().fold(Dim::NONE, |d, (u, e)| {
            d.mul(&u.dim().pow(*e).unwrap_or(Dim::NONE)).unwrap_or(d)
        })
    }

    pub fn mul(&self, o: &Self) -> Self {
        Self::from_terms(self.0.iter().chain(&o.0).copied())
    }

    pub fn recip(&self) -> Self {
        Self::from_terms(
            self.0
                .iter()
                .map(|(u, e)| (*u, e.checked_neg().unwrap_or(*e))),
        )
    }

    pub fn div(&self, o: &Self) -> Self {
        self.mul(&o.recip())
    }

    pub fn pow(&self, p: Rational) -> Option<Self> {
        let mut terms = Vec::new();
        for (u, e) in &self.0 {
            terms.push((*u, e.checked_mul(p)?));
        }
        Some(Self::from_terms(terms))
    }

    /// Chooses display units for a product without changing its value.
    ///
    /// 1. Repeated units merge (`m·m` → `m²`).
    /// 2. Units of the same base dimension merge into the first
    ///    (`m·cm` → `m²`, `h/min` → dimensionless).
    /// 3. Compound units expand when that lets terms cancel (`mph·h` → `mi`).
    /// 4. An all-SI product with a named derived unit becomes that unit
    ///    (`kg·m/s²` → `N`).
    pub fn simplify(&self) -> Self {
        let merged = self.unify_bases();
        let expanded = Self::from_terms(merged.0.iter().flat_map(|(u, e)| {
            match u.expansion().and_then(|x| x.pow(*e)) {
                Some(x) => x.0,
                None => vec![(*u, *e)],
            }
        }))
        .unify_bases();
        let best = if expanded.len() < merged.len() {
            expanded
        } else {
            merged
        };
        if best.len() >= 2 && best.0.iter().all(|(u, _)| u.system() == table::System::Si) {
            if let Some(d) = quantity::quantity_for(&best.dim()).and_then(|q| q.derived.clone()) {
                return d;
            }
        }
        best
    }

    fn unify_bases(&self) -> Self {
        let mut out: Vec<(UnitRef, Rational)> = Vec::new();
        for (u, e) in &self.0 {
            let base = u.dim().as_base();
            let slot = out.iter_mut().find(|(v, _)| {
                *v == *u || (base.is_some() && v.dim().as_base() == base && !v.is_affine())
            });
            match slot {
                Some((_, acc)) if !u.is_affine() => *acc = acc.checked_add(*e).unwrap_or(*acc),
                _ => out.push((*u, *e)),
            }
        }
        out.retain(|(_, e)| !e.is_zero());
        Self(out)
    }

    /// `km/h`, `kg·m²/s³`, `s⁻¹`; or `kg*m^2/s^3` when `fancy` is false.
    pub fn display(&self, fancy: bool) -> String {
        let sep = if fancy { "·" } else { "*" };
        let term = |u: &UnitRef, e: Rational| {
            if e == Rational::ONE {
                u.symbol()
            } else {
                format!("{}{}", u.symbol(), exponent(e, fancy))
            }
        };
        let num: Vec<String> = self
            .0
            .iter()
            .filter(|(_, e)| !e.is_negative())
            .map(|(u, e)| term(u, *e))
            .collect();
        let den: Vec<String> = self
            .0
            .iter()
            .filter(|(_, e)| e.is_negative())
            .map(|(u, e)| term(u, e.abs()))
            .collect();
        match (num.is_empty(), den.len()) {
            (_, 0) => num.join(sep),
            (true, _) => self
                .0
                .iter()
                .map(|(u, e)| format!("{}{}", u.symbol(), exponent(*e, fancy)))
                .collect::<Vec<_>>()
                .join(sep),
            (false, 1) => format!("{}/{}", num.join(sep), den[0]),
            (false, _) => format!("{}/({})", num.join(sep), den.join(sep)),
        }
    }
}

/// A parsed definition: a coefficient times units.
#[derive(Debug)]
pub struct Parsed {
    pub coef: Number,
    pub terms: Vec<(UnitRef, Rational)>,
}

/// Parses a definition from data/units.toml: `0.3048 m`, `pi/180 rad`,
/// `kg m/s^2`, `(m/s)^2`, `1/3 tbsp`. Juxtaposition multiplies.
pub fn parse_def(s: &str) -> Result<Parsed, String> {
    let toks = tokenize(s)?;
    let mut p = DefParser { toks, pos: 0 };
    let out = p.product()?;
    if p.pos != p.toks.len() {
        return Err(format!("unexpected {:?} in {s:?}", p.toks[p.pos]));
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(String),
    Word(String),
    Op(char),
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if "*/^()-".contains(c) {
            out.push(Tok::Op(c));
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let mut j = i + 1;
                if j < chars.len() && (chars[j] == '-' || chars[j] == '+') {
                    j += 1;
                }
                if j < chars.len() && chars[j].is_ascii_digit() {
                    i = j;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            out.push(Tok::Num(chars[start..i].iter().collect()));
        } else {
            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() && !"*/^()".contains(chars[i]) {
                i += 1;
            }
            out.push(Tok::Word(chars[start..i].iter().collect()));
        }
    }
    Ok(out)
}

struct DefParser {
    toks: Vec<Tok>,
    pos: usize,
}

impl DefParser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn product(&mut self) -> Result<Parsed, String> {
        let mut acc = self.power()?;
        loop {
            let divide = match self.peek() {
                Some(Tok::Op('*')) => {
                    self.pos += 1;
                    false
                }
                Some(Tok::Op('/')) => {
                    self.pos += 1;
                    true
                }
                Some(Tok::Num(_) | Tok::Word(_) | Tok::Op('(')) => false,
                _ => return Ok(acc),
            };
            let rhs = self.power()?;
            acc = combine(acc, rhs, divide)?;
        }
    }

    fn power(&mut self) -> Result<Parsed, String> {
        let base = self.atom()?;
        if self.peek() != Some(&Tok::Op('^')) {
            return Ok(base);
        }
        self.pos += 1;
        let e = self.exponent()?;
        let coef = base
            .coef
            .pow(Number::exact(e, false))
            .map_err(|e| format!("{e:?}"))?;
        let terms = base
            .terms
            .into_iter()
            .map(|(u, x)| x.checked_mul(e).map(|x| (u, x)))
            .collect::<Option<Vec<_>>>()
            .ok_or("exponent overflow")?;
        Ok(Parsed { coef, terms })
    }

    fn exponent(&mut self) -> Result<Rational, String> {
        let neg = if self.peek() == Some(&Tok::Op('-')) {
            self.pos += 1;
            true
        } else {
            false
        };
        let r = match self.toks.get(self.pos).cloned() {
            Some(Tok::Num(n)) => {
                self.pos += 1;
                Rational::parse(&n).ok_or("bad exponent")?
            }
            Some(Tok::Op('(')) => {
                self.pos += 1;
                let mut text = String::new();
                while let Some(t) = self.toks.get(self.pos).cloned() {
                    self.pos += 1;
                    match t {
                        Tok::Op(')') => break,
                        Tok::Num(n) => text.push_str(&n),
                        Tok::Op(c) => text.push(c),
                        Tok::Word(w) => return Err(format!("bad exponent {w}")),
                    }
                }
                Rational::parse(&text).ok_or("bad exponent")?
            }
            other => return Err(format!("expected exponent, found {other:?}")),
        };
        Ok(if neg {
            r.checked_neg().ok_or("overflow")?
        } else {
            r
        })
    }

    fn atom(&mut self) -> Result<Parsed, String> {
        let tok = self.toks.get(self.pos).cloned().ok_or("unexpected end")?;
        self.pos += 1;
        match tok {
            Tok::Num(n) => {
                // Exact when it fits in a rational; 3.2e-53 falls back to f64.
                let coef = match Rational::parse(&n) {
                    Some(r) => Number::exact(r, true),
                    None => n
                        .parse::<f64>()
                        .ok()
                        .and_then(|x| Number::approx(x).ok())
                        .ok_or("bad number")?,
                };
                Ok(Parsed {
                    coef,
                    terms: vec![],
                })
            }
            Tok::Word(w) if w == "pi" => Ok(Parsed {
                coef: Number::Approx(std::f64::consts::PI),
                terms: vec![],
            }),
            Tok::Word(w) => {
                let u = table::lookup(&w).ok_or_else(|| format!("unknown unit {w:?}"))?;
                Ok(Parsed {
                    coef: Number::ONE,
                    terms: vec![(u, Rational::ONE)],
                })
            }
            // -4.66e-4 (negative constants in CODATA)
            Tok::Op('-') => {
                let inner = self.atom()?;
                Ok(Parsed {
                    coef: inner.coef.neg(),
                    terms: inner.terms,
                })
            }
            Tok::Op('(') => {
                let inner = self.product()?;
                if self.toks.get(self.pos) != Some(&Tok::Op(')')) {
                    return Err("missing )".into());
                }
                self.pos += 1;
                Ok(inner)
            }
            Tok::Op(c) => Err(format!("unexpected {c}")),
        }
    }
}

fn combine(a: Parsed, b: Parsed, divide: bool) -> Result<Parsed, String> {
    let coef = if divide {
        a.coef.div(b.coef)
    } else {
        a.coef.mul(b.coef)
    };
    let mut terms = a.terms;
    for (u, e) in b.terms {
        terms.push((
            u,
            if divide {
                e.checked_neg().ok_or("overflow")?
            } else {
                e
            },
        ));
    }
    Ok(Parsed {
        coef: coef.map_err(|e| format!("{e:?}"))?,
        terms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> UnitExpr {
        UnitExpr::parse(s).unwrap()
    }

    #[test]
    fn display() {
        assert_eq!(u("km/h").display(true), "km/h");
        assert_eq!(u("kg m^2/s^3").display(true), "kg·m²/s³");
        assert_eq!(u("kg m^2/s^3").display(false), "kg*m^2/s^3");
        assert_eq!(u("J/kg K").display(true), "J·K/kg");
        assert_eq!(u("1/d").display(true), "d⁻¹");
        assert_eq!(u("m^(1/2)").display(true), "m^(1/2)");
        assert_eq!(u("W/(m^2 K)").display(true), "W/(m²·K)");
    }

    #[test]
    fn simplify() {
        assert_eq!(u("mph h").simplify(), u("mi"));
        assert_eq!(u("m cm").simplify(), u("m^2"));
        assert!(u("h/min").simplify().is_empty());
        assert_eq!(u("kg m/s^2").simplify(), u("N"));
        assert_eq!(u("kW h").simplify(), u("kW h"), "h is not SI, so no J");
        assert_eq!(u("m/s").simplify(), u("m/s"));
    }

    #[test]
    fn factors() {
        assert_eq!(u("km/h").factor().as_rational(), Rational::new(5, 18));
        assert_eq!(
            u("ft^2").factor().as_rational(),
            Rational::parse("0.09290304")
        );
    }
}
