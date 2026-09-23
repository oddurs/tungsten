//! The "for scale" pod (docs/concept.md §5).
//!
//! ```text
//!   ◆ for scale
//!   │ ≈ 1 Olympic pool
//!   │ ≈ 2 Lake Superiors
//! ```
//!
//! A result is compared to every reference the knowledge base tags for scale
//! (`scale = [...]` in data/items.toml and food.toml) whose property has the
//! result's dimension. A comparison counts only if the ratio lies between ½
//! and 1000; among those, ratios close to a round number win, then smaller
//! ones. At most two are shown, never two from the same domain, and never the
//! thing the query was about.

use crate::{Body, Line, NumMode, Pod, Seg};
use tungsten_core::{Entity, Outcome, Source, Value};
use tungsten_kb::entities;
use tungsten_units::{Number, Rational, UnitExpr};

const MIN: f64 = 0.5;
const MAX: f64 = 1000.0;
const SHOWN: usize = 2;

/// How far `r` is from a round number: 1, 2, 2.5, 3, 5 or 7.5 times a power
/// of ten. 0 is perfectly round.
fn roughness(r: f64) -> f64 {
    let exp = r.log10().floor();
    let scale = 10f64.powf(exp);
    let m = r / scale;
    [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 7.5, 10.0]
        .iter()
        .map(|nice| ((m - nice) / nice).abs())
        .fold(f64::INFINITY, f64::min)
}

struct Comparison {
    entity: Entity,
    prop: tungsten_kb::Prop,
    ratio: f64,
    score: f64,
}

pub fn pod(o: &Outcome) -> Option<Pod> {
    let v: &Value = &o.value;
    if v.dim.is_none() || v.point {
        return None;
    }
    let x = v.num.to_f64();
    if x <= 0.0 || !x.is_finite() {
        return None;
    }
    let about: Vec<Entity> = o
        .sources
        .iter()
        .filter_map(|s| match s {
            Source::Value { entity, .. } | Source::Entity(entity) => Some(*entity),
            Source::Unit(_) => None,
        })
        .collect();

    let mut found: Vec<Comparison> = Vec::new();
    for e in entities().filter(|e| !about.contains(e)) {
        for p in e.scale() {
            let Some(val) = e.value(p) else { continue };
            let Ok(q) = tungsten_core::quantity(val.num, &val.unit) else {
                continue;
            };
            if q.dim != v.dim {
                continue;
            }
            let ratio = x / q.num.to_f64();
            if !(MIN..=MAX).contains(&ratio) {
                continue;
            }
            // Round ratios read best; small ones are easier to picture.
            let score = roughness(ratio) + ratio.log10().max(0.0) * 0.02;
            found.push(Comparison {
                entity: e,
                prop: p,
                ratio,
                score,
            });
        }
    }
    found.sort_by(|a, b| a.score.total_cmp(&b.score));

    let mut picked: Vec<Comparison> = Vec::new();
    for c in found {
        let same_domain = picked
            .iter()
            .any(|p| p.entity == c.entity || p.entity.domain() == c.entity.domain());
        if !same_domain {
            picked.push(c);
        }
        if picked.len() == SHOWN {
            break;
        }
    }
    if picked.is_empty() {
        return None;
    }
    let lines = picked.iter().map(line).collect();
    Some(Pod {
        title: "for scale".into(),
        error: false,
        body: Body::Lines(lines),
    })
}

/// `≈ 3 bananas`, `≈ 1 Olympic pool`, `≈ 0.6 × blue whale`.
fn line(c: &Comparison) -> Line {
    let r = c.ratio;
    // Two significant figures is as much as a comparison deserves.
    let digits = if r >= 10.0 { 0 } else { 1 };
    let magnitude = 10f64.powf(r.log10().floor() - 1.0).max(1.0);
    let rounded = if r >= 100.0 {
        (r / magnitude).round() * magnitude
    } else {
        let f = 10f64.powi(digits);
        (r * f).round() / f
    };
    let num = Rational::parse(&format!("{rounded}"))
        .map_or(Number::Approx(rounded), |q| Number::exact(q, true));
    let mut segs = vec![Seg::Dim("≈ ".into())];
    if rounded < 1.0 {
        segs.push(Seg::Value {
            num,
            unit: UnitExpr::default(),
            mode: NumMode::Result,
        });
        segs.push(Seg::Text(format!(" × {}", c.entity.display())));
    } else {
        let one = (rounded - 1.0).abs() < f64::EPSILON;
        segs.push(Seg::Value {
            num,
            unit: UnitExpr::default(),
            mode: NumMode::Result,
        });
        let name = if one {
            c.entity.display()
        } else {
            c.entity.plural()
        };
        segs.push(Seg::Text(format!(" {name}")));
    }
    // `≈ 2 Moons (radius)`: say which property when it could be several.
    if c.entity.scale().count() > 1 && c.entity.default() != Some(c.prop) {
        segs.push(Seg::Dim(format!(" ({})", c.prop.name())));
    }
    Line(segs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_numbers_are_smooth() {
        assert!(roughness(1.0) < 1e-9);
        assert!(roughness(250.0) < 1e-9);
        assert!(roughness(3.3) > roughness(3.0));
        assert!(roughness(7.1) > roughness(7.5));
    }

    fn scale_of(q: &str) -> Vec<String> {
        let o = tungsten_core::evaluate(q).unwrap();
        let Some(p) = pod(&o) else { return vec![] };
        let Body::Lines(lines) = p.body else { panic!() };
        lines
            .iter()
            .map(|l| {
                l.0.iter()
                    .map(|s| match s {
                        Seg::Text(t) | Seg::Dim(t) => t.clone(),
                        Seg::Value { num, .. } => format!("{}", num.to_f64()),
                        _ => String::new(),
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn olympic_pool() {
        let s = scale_of("2.5 million L");
        assert_eq!(s[0], "≈ 1 Olympic pool", "{s:?}");
    }

    #[test]
    fn never_outside_half_to_a_thousand() {
        for q in [
            "1 mm", "1 cm", "1 km", "1 g", "1 kg", "1 t", "1 m^2", "1 L", "1 GB", "1 yr", "1e9 kg",
        ] {
            for line in scale_of(q) {
                let n: f64 = line
                    .trim_start_matches("≈ ")
                    .split(' ')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!((0.5..=1000.0).contains(&n), "{q}: {line}");
            }
        }
    }

    #[test]
    fn not_compared_to_itself() {
        let s = scale_of("volume of olympic pool");
        assert!(s.iter().all(|l| !l.contains("Olympic pool")), "{s:?}");
    }

    #[test]
    fn about_one_is_one() {
        assert_eq!(scale_of("70 yr")[0], "≈ 1 human lifetime");
    }

    #[test]
    fn two_domains_at_most_one_each() {
        let s = scale_of("100 t");
        assert!(s.len() <= 2);
    }
}
