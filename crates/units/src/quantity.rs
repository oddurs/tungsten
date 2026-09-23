//! Named quantities: what a dimension is called, and the units worth offering
//! for it.

use crate::table::{self, QUANTITIES};
use crate::{BaseDim, Dim, UnitExpr};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Quantity {
    pub name: &'static str,
    pub dim: Dim,
    /// The SI unit to show this quantity in: `N` for force, `m/s` for velocity.
    pub coherent: UnitExpr,
    /// A named SI unit that products of SI units collapse into.
    pub derived: Option<UnitExpr>,
    /// Each group offers its best candidate in the "other units" pod.
    pub groups: Vec<Vec<UnitExpr>>,
    /// Consulted only when the result is already in one of the group's units.
    pub extra: Vec<Vec<UnitExpr>>,
}

pub fn quantities() -> &'static [Quantity] {
    static TABLE: OnceLock<Vec<Quantity>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let parse = |s: &str| UnitExpr::parse(s).unwrap_or_else(|| panic!("bad unit {s:?}"));
        let groups = |g: &[&[&str]]| -> Vec<Vec<UnitExpr>> {
            g.iter()
                .map(|v| v.iter().map(|s| parse(s)).collect())
                .collect()
        };
        QUANTITIES
            .iter()
            .map(|q| {
                let coherent = parse(q.dim);
                Quantity {
                    name: q.name,
                    dim: coherent.dim(),
                    coherent,
                    derived: q.derived.map(parse),
                    groups: groups(q.groups),
                    extra: groups(q.extra),
                }
            })
            .collect()
    })
}

pub fn quantity_for(dim: &Dim) -> Option<&'static Quantity> {
    quantities().iter().find(|q| q.dim == *dim)
}

/// `velocity`, or composed from base names: `length²·time⁻¹`.
pub fn describe(dim: &Dim, fancy: bool) -> String {
    match quantity_for(dim) {
        Some(q) => q.name.to_string(),
        None => dim.compose(fancy),
    }
}

/// The SI unit expression for a dimension: a named quantity's unit, else the
/// product of base units.
pub fn coherent(dim: &Dim) -> UnitExpr {
    if let Some(q) = quantity_for(dim) {
        return q.coherent.clone();
    }
    let terms = BaseDim::ALL.iter().filter_map(|b| {
        let e = dim.exponent(*b);
        if e.is_zero() {
            return None;
        }
        let sym = match b {
            BaseDim::Length => "m",
            BaseDim::Mass => "kg",
            BaseDim::Time => "s",
            BaseDim::Current => "A",
            BaseDim::Temperature => "K",
            BaseDim::Amount => "mol",
            BaseDim::Luminosity => "cd",
            BaseDim::Currency => "USD",
            BaseDim::Information => "bit",
            BaseDim::Count => return None,
        };
        table::lookup_symbol(sym).map(|u| (u, e))
    });
    UnitExpr::from_terms(terms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        let v = UnitExpr::parse("km/h").unwrap().dim();
        assert_eq!(describe(&v, true), "velocity");
        let odd = UnitExpr::parse("m^2/s").unwrap().dim();
        assert_eq!(describe(&odd, true), "length²·time⁻¹");
        assert_eq!(coherent(&odd).display(true), "m²/s");
        let force = UnitExpr::parse("lbf").unwrap().dim();
        assert_eq!(coherent(&force).display(true), "N");
    }
}
