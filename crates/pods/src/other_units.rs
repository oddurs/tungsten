//! Choosing the units for the "other units" pod (docs/concept.md §5).
//!
//! Each group of a quantity's candidate units (metric, customary, …)
//! contributes the candidate whose value reads most naturally: between 1 and
//! 1000 is ideal, anything from about 0.0003 to three million is acceptable. When no
//! group has an acceptable candidate, the quantity's extra groups (nautical,
//! astronomical, …) are tried. The SI unit is always offered last.

use tungsten_core::{Value, is_scale};
use tungsten_units::{BaseDim, Dim, Number, UnitExpr, coherent, quantity_for};

const MAX: usize = 4;
const ACCEPTABLE: f64 = 3.5;

/// How far `x` is from reading naturally, in decades. 0 means 1 ≤ x < 1000.
fn badness(x: f64) -> f64 {
    let x = x.abs();
    if x == 0.0 || !x.is_finite() {
        f64::INFINITY
    } else if x < 1.0 {
        -x.log10()
    } else if x >= 1000.0 {
        (x / 1000.0).log10() + 0.001
    } else {
        0.0
    }
}

/// For a difference, an affine unit is shown as its Δ unit.
fn fit(v: &Value, u: &UnitExpr) -> UnitExpr {
    match u.single() {
        Some(r) if !v.point && r.is_affine() => r.delta().map_or_else(|| u.clone(), UnitExpr::one),
        _ => u.clone(),
    }
}

pub fn other_units(v: &Value, shown: &UnitExpr) -> Vec<(Number, UnitExpr)> {
    if v.dim.is_none() || v.num.is_zero() {
        return Vec::new();
    }
    let mut picks: Vec<UnitExpr> = Vec::new();
    let taken = |picks: &[UnitExpr], u: &UnitExpr| u == shown || picks.contains(u);

    if v.point && v.dim == Dim::base(BaseDim::Temperature) {
        // Temperatures always get every scale.
        for s in ["°C", "°F", "K"] {
            if let Some(u) = UnitExpr::parse(s).filter(|u| u.single().is_some_and(is_scale))
                && !taken(&picks, &u)
            {
                picks.push(u);
            }
        }
    } else if let Some(q) = quantity_for(&v.dim) {
        let best = |group: &[UnitExpr], picks: &[UnitExpr]| {
            group
                .iter()
                .map(|u| fit(v, u))
                .filter(|u| !taken(picks, u))
                .filter_map(|u| v.in_unit(&u).ok().map(|n| (badness(n.to_f64()), u)))
                .filter(|(b, _)| *b <= ACCEPTABLE)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, u)| u)
        };
        for g in &q.groups {
            if let Some(u) = best(g, &picks) {
                picks.push(u);
            }
        }
        for g in &q.extra {
            // Extra groups speak when spoken to, or when nothing else fits.
            if (g.contains(shown) || picks.is_empty())
                && let Some(u) = best(g, &picks)
            {
                picks.push(u);
            }
        }
    }

    let si = fit(v, &coherent(&v.dim));
    if !si.is_empty() && !taken(&picks, &si) {
        picks.push(si);
    }
    picks.truncate(MAX);
    picks
        .into_iter()
        .filter_map(|u| v.in_unit(&u).ok().map(|n| (n, u)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tungsten_core::{Answer, evaluate};

    fn alts(q: &str) -> Vec<String> {
        let o = evaluate(q).unwrap();
        let Answer::Single { unit, .. } = &o.answer else {
            panic!()
        };
        other_units(&o.value, unit)
            .into_iter()
            .map(|(n, u)| format!("{:.4} {}", n.to_f64(), u.display(false)))
            .collect()
    }

    #[test]
    fn length() {
        assert_eq!(
            alts("135 mi"),
            ["217.2614 km", "237600.0000 yd", "217261.4400 m"]
        );
    }

    #[test]
    fn astronomical_when_nothing_else_fits() {
        let a = alts("speed of light * 1 week in m");
        assert_eq!(a[0], "1212.0124 au");
    }

    #[test]
    fn temperatures_get_every_scale() {
        assert_eq!(alts("37 °C"), ["98.6000 °F", "310.1500 K"]);
        let d = alts("30 °C - 20 °C");
        assert!(d.iter().any(|s| s.ends_with("Δ°F")), "{d:?}");
    }

    #[test]
    fn dimensionless_has_none() {
        assert!(alts("1/3 + 1/6").is_empty());
    }
}
