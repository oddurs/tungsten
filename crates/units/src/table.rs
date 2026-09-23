//! The compiled unit table (generated from data/units.toml) and lookup.

use crate::{BaseDim, Dim, Number, Rational, UnitExpr};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct PrefixDef {
    pub symbol: &'static str,
    pub name: &'static str,
    pub factor: &'static str,
    pub source: &'static str,
}

#[derive(Debug)]
pub enum Def {
    Base { dim: BaseDim, scale: &'static str },
    Expr(&'static str),
}

/// Which measurement system a unit belongs to; drives "other units" choices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    Si,
    Metric,
    Imperial,
    Us,
    Nautical,
    Astronomical,
    Information,
    Physics,
    Other,
}

impl System {
    /// SI and the units accepted alongside it.
    pub fn is_metric(self) -> bool {
        matches!(self, Self::Si | Self::Metric)
    }

    /// US customary and British imperial.
    pub fn is_customary(self) -> bool {
        matches!(self, Self::Imperial | Self::Us)
    }
}

#[derive(Debug)]
pub struct UnitDef {
    pub name: &'static str,
    pub plural: &'static str,
    pub display: &'static str,
    pub def: Def,
    pub offset: Option<&'static str>,
    pub delta: Option<&'static str>,
    pub prefixes: u64,
    pub expand: bool,
    pub system: System,
    pub source: &'static str,
}

#[derive(Debug)]
pub struct QuantityDef {
    pub name: &'static str,
    pub dim: &'static str,
    pub derived: Option<&'static str>,
    pub groups: &'static [&'static [&'static str]],
    pub extra: &'static [&'static [&'static str]],
}

include!(concat!(env!("OUT_DIR"), "/units.rs"));

const NO_PREFIX: u8 = u8::MAX;

/// A unit, possibly prefixed: `km` is (metre, kilo).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct UnitRef {
    unit: u16,
    prefix: u8,
}

/// A unit's size in SI base units, and its dimension.
#[derive(Clone, Copy, Debug)]
struct Resolved {
    factor: Number,
    dim: Dim,
}

fn resolved() -> &'static [Resolved] {
    static TABLE: OnceLock<Vec<Resolved>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut state: Vec<Option<Resolved>> = vec![None; UNITS.len()];
        let mut visiting = vec![false; UNITS.len()];
        for (i, u) in UNITS.iter().enumerate() {
            resolve(i, &mut state, &mut visiting)
                .unwrap_or_else(|e| panic!("unit {:?}: {e}", u.name));
        }
        state.into_iter().map(|r| r.expect("resolved")).collect()
    })
}

fn resolve(
    i: usize,
    state: &mut Vec<Option<Resolved>>,
    visiting: &mut Vec<bool>,
) -> Result<Resolved, String> {
    if let Some(r) = state[i] {
        return Ok(r);
    }
    if visiting[i] {
        return Err("definition is circular".into());
    }
    visiting[i] = true;
    let r = match UNITS[i].def {
        Def::Base { dim, scale } => Resolved {
            factor: Number::exact(Rational::parse(scale).ok_or("bad scale")?, true),
            dim: Dim::base(dim),
        },
        Def::Expr(expr) => {
            let parsed = crate::expr::parse_def(expr)?;
            let mut factor = parsed.coef;
            let mut dim = Dim::NONE;
            for (u, e) in parsed.terms {
                let dep = resolve(u.unit as usize, state, visiting)?;
                let f = u
                    .prefix_factor()
                    .mul(dep.factor)
                    .map_err(|e| format!("{e:?}"))?;
                let f = f
                    .pow(Number::exact(e, false))
                    .map_err(|e| format!("{e:?}"))?;
                factor = factor.mul(f).map_err(|e| format!("{e:?}"))?;
                dim = dim
                    .mul(&dep.dim.pow(e).ok_or("exponent overflow")?)
                    .ok_or("overflow")?;
            }
            Resolved {
                factor: factor.with_decimal(true),
                dim,
            }
        }
    };
    visiting[i] = false;
    state[i] = Some(r);
    Ok(r)
}

fn find(table: &'static [(&'static str, u16, u8)], key: &str) -> Option<UnitRef> {
    table
        .binary_search_by(|(k, _, _)| (*k).cmp(key))
        .ok()
        .map(|i| UnitRef {
            unit: table[i].1,
            prefix: table[i].2,
        })
}

/// Looks a word up as a symbol (case-sensitive) or a name (case-insensitive).
pub fn lookup(word: &str) -> Option<UnitRef> {
    lookup_symbol(word).or_else(|| lookup_name(word))
}

pub fn lookup_symbol(word: &str) -> Option<UnitRef> {
    find(SYMBOLS, word)
}

pub fn lookup_name(word: &str) -> Option<UnitRef> {
    find(NAMES, &word.to_lowercase())
}

/// Every symbol and name, for did-you-mean suggestions.
pub fn all_forms() -> impl Iterator<Item = &'static str> {
    SYMBOLS.iter().chain(NAMES).map(|(k, _, _)| *k)
}

/// Longest multi-word name, in words, so a resolver knows how far to look.
pub fn max_name_words() -> usize {
    NAMES
        .iter()
        .map(|(k, _, _)| k.split(' ').count())
        .max()
        .unwrap_or(1)
}

pub fn unit_count() -> usize {
    UNITS.len()
}

impl UnitRef {
    pub fn def(self) -> &'static UnitDef {
        &UNITS[self.unit as usize]
    }

    pub fn prefix(self) -> Option<&'static PrefixDef> {
        (self.prefix != NO_PREFIX).then(|| &PREFIXES[self.prefix as usize])
    }

    fn prefix_factor(self) -> Number {
        match self.prefix() {
            Some(p) => Number::exact(Rational::parse(p.factor).expect("prefix factor"), true),
            None => Number::ONE,
        }
    }

    /// Size in SI base units. Always decimal-flavoured.
    pub fn factor(self) -> Number {
        let base = resolved()[self.unit as usize].factor;
        self.prefix_factor()
            .mul(base)
            .unwrap_or(base)
            .with_decimal(true)
    }

    pub fn dim(self) -> Dim {
        resolved()[self.unit as usize].dim
    }

    /// For affine scales (°C, °F): the kelvin value of this scale's zero.
    pub fn offset(self) -> Option<Number> {
        if self.prefix().is_some() {
            return None;
        }
        self.def()
            .offset
            .map(|o| Number::exact(Rational::parse(o).expect("offset"), true))
    }

    pub fn is_affine(self) -> bool {
        self.offset().is_some()
    }

    /// The difference unit for an affine scale: °C → Δ°C.
    pub fn delta(self) -> Option<Self> {
        self.def().delta.and_then(lookup_symbol)
    }

    pub fn system(self) -> System {
        self.def().system
    }

    pub fn symbol(self) -> String {
        match self.prefix() {
            Some(p) => format!("{}{}", p.symbol, self.def().display),
            None => self.def().display.to_string(),
        }
    }

    pub fn name(self, plural: bool) -> String {
        let d = self.def();
        let base = if plural { d.plural } else { d.name };
        match self.prefix() {
            Some(p) => format!("{}{base}", p.name),
            None => base.to_string(),
        }
    }

    pub fn source(self) -> &'static str {
        self.def().source
    }

    /// For compound units marked `expand` (mph → mi/h), the expansion.
    pub fn expansion(self) -> Option<UnitExpr> {
        let d = self.def();
        if !d.expand || self.prefix().is_some() {
            return None;
        }
        match d.def {
            Def::Expr(e) => {
                let p = crate::expr::parse_def(e).ok()?;
                (p.coef.to_f64() == 1.0).then(|| UnitExpr::from_terms(p.terms))
            }
            Def::Base { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_unit_resolves() {
        for (i, u) in UNITS.iter().enumerate() {
            let r = resolved()[i];
            assert!(
                r.factor.to_f64() > 0.0,
                "{} has non-positive factor",
                u.name
            );
        }
    }

    #[test]
    fn every_quantity_resolves() {
        for q in QUANTITIES {
            let dim = UnitExpr::parse(q.dim).unwrap_or_else(|| panic!("{}: bad dim", q.name));
            for g in q.groups.iter().chain(q.extra) {
                for s in *g {
                    let u =
                        UnitExpr::parse(s).unwrap_or_else(|| panic!("{}: bad unit {s}", q.name));
                    assert_eq!(
                        u.dim(),
                        dim.dim(),
                        "{}: {s} has the wrong dimension",
                        q.name
                    );
                }
            }
            if let Some(d) = q.derived {
                assert_eq!(UnitExpr::parse(d).unwrap().dim(), dim.dim());
            }
        }
    }

    #[test]
    fn every_entry_has_a_source() {
        assert!(UNITS.iter().all(|u| !u.source.trim().is_empty()));
        assert!(PREFIXES.iter().all(|p| !p.source.trim().is_empty()));
    }

    #[test]
    fn lookups() {
        let km = lookup("km").unwrap();
        assert_eq!(km.symbol(), "km");
        assert_eq!(km.name(true), "kilometres");
        assert_eq!(lookup("Kilometers"), Some(km));
        assert_eq!(lookup("feet").unwrap().symbol(), "ft");
        assert_eq!(lookup("fl oz").unwrap().symbol(), "fl oz");
        assert_eq!(lookup("µs"), lookup("us"));
        assert!(lookup("KM").is_none());
        assert_eq!(lookup("MiB").unwrap().name(false), "mebibyte");
    }

    #[test]
    fn exact_factors() {
        let mi = lookup("mi").unwrap().factor();
        assert_eq!(mi.as_rational(), Rational::parse("1609.344"));
        let f = lookup("°F").unwrap();
        assert_eq!(f.factor().as_rational(), Rational::new(5, 9));
        assert_eq!(f.offset().unwrap().as_rational(), Rational::new(45967, 180));
        let ly = lookup("ly").unwrap().factor();
        assert_eq!(ly.as_rational(), Some(Rational::int(9_460_730_472_580_800)));
    }

    #[test]
    fn counts() {
        // Reported in cairn item 0015; keep the numbers honest.
        assert!(UNITS.len() >= 100, "{} units", UNITS.len());
        assert!(SYMBOLS.len() + NAMES.len() > 2000);
    }
}
