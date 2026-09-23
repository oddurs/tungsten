//! Entity cards: what a lone name shows (docs/concept.md §5).
//!
//! ```text
//!   ◆ gold · Au · 79
//!   │ atomic mass    196.96657 u
//!   │ density        19.282 g/cm³
//!   │ melting point  1064.18 °C
//! ```

use crate::{Body, Line, NumMode, Pod, Seg};
use tungsten_kb::{Entity, Kind};
use tungsten_units::{UnitExpr, parse_def};

pub fn title(e: Entity) -> String {
    match e.kind() {
        Kind::Element => {
            let z = e
                .values()
                .find(|v| v.prop.name() == "atomic number")
                .map(|v| format!(" · {}", v.num.to_f64()));
            format!(
                "{} · {}{}",
                e.display(),
                e.symbol().unwrap_or(""),
                z.unwrap_or_default()
            )
        }
        Kind::Constant => match e.symbol() {
            Some(s) => format!("{} · {s}", e.display()),
            None => e.display().to_string(),
        },
        Kind::Item => e.display().to_string(),
        k => format!("{} · {}", e.display(), k.name()),
    }
}

fn label(name: &str, width: usize) -> Seg {
    Seg::Dim(format!("{name:<width$}  "))
}

pub fn card(e: Entity) -> Pod {
    let mut values: Vec<_> = e
        .values()
        .filter(|v| !(e.kind() == Kind::Element && v.prop.name() == "atomic number"))
        .collect();
    // Card order is declaration order in data/properties.toml.
    values.sort_by_key(|v| v.prop);
    let facts: Vec<_> = e.facts().collect();
    let width = values
        .iter()
        .map(|v| v.prop.name().len())
        .chain(facts.iter().map(|f| f.name.len()))
        .chain(
            values
                .iter()
                .filter(|v| v.uncertainty.is_some())
                .map(|_| "uncertainty".len()),
        )
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    for v in &values {
        let mut num = v.num;
        let mut unit = v.unit.clone();
        if let Some(show) = v.prop.show().filter(|s| s.dim() == v.unit.dim()) {
            let value = tungsten_core::quantity(v.num, &v.unit).ok();
            if let Some(n) = value.and_then(|x| x.in_unit(&show).ok()) {
                num = n;
                unit = show;
            }
        }
        // Shown in its own unit, a value keeps the digits its source published.
        let mode = match v.digits {
            Some(d) if unit == v.unit => NumMode::Published(d),
            _ => NumMode::Result,
        };
        lines.push(Line(vec![
            label(v.prop.name(), width),
            Seg::Value { num, unit, mode },
        ]));
        match v.uncertainty {
            Some("exact") => lines.push(Line(vec![
                label("uncertainty", width),
                Seg::Dim("exact".into()),
            ])),
            Some(u) => {
                if let Ok(p) = parse_def(u) {
                    lines.push(Line(vec![
                        label("uncertainty", width),
                        Seg::Text("± ".into()),
                        Seg::Value {
                            num: p.coef,
                            unit: UnitExpr::from_terms(p.terms),
                            mode: NumMode::Result,
                        },
                    ]));
                }
            }
            None => {}
        }
    }
    for f in &facts {
        lines.push(Line(vec![label(f.name, width), Seg::Text(f.text.into())]));
    }
    Pod {
        title: title(e),
        error: false,
        body: Body::Lines(lines),
    }
}
