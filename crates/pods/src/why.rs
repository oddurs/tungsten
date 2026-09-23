//! The sources pod, for `--why` (docs/concept.md §4).
//!
//! ```text
//!   ◆ sources
//!   │ Earth · mass  NASA NSSDCA Earth Fact Sheet (…), updated 15 November 2024
//!   │ kg            BIPM SI Brochure (9th ed., 2019), §2.3.1, Table 2
//! ```

use crate::{Body, Line, Pod, Seg};
use tungsten_core::Source;

/// Short citation codes used in data/units.toml, and what they stand for.
const CODES: &[(&str, &str)] = &[
    ("BIPM", "BIPM SI Brochure (9th ed., 2019)"),
    ("CGPM22", "27th CGPM (2022), Resolution 3"),
    ("SP811", "NIST Special Publication 811 (2008)"),
    ("HB44", "NIST Handbook 44 (2024), Appendix C"),
    ("IEC 80000-13", "IEC 80000-13:2008"),
    ("ISO 80000-3", "ISO 80000-3:2019"),
    ("WMA", "UK Weights and Measures Act 1985"),
];

/// `HB44 C-4` → `NIST Handbook 44 (2024), Appendix C, C-4`.
fn expand(source: &str) -> String {
    for (code, full) in CODES {
        if let Some(rest) = source.strip_prefix(code) {
            if rest.is_empty() {
                return (*full).to_string();
            }
            if let Some(rest) = rest.strip_prefix(' ') {
                return format!("{full}, {rest}");
            }
        }
    }
    source.to_string()
}

fn row(source: &Source) -> Vec<(String, String)> {
    match source {
        Source::Unit(u) => {
            let what = if u.prefix().is_some() {
                format!("{} ({})", u.symbol(), u.def().display)
            } else {
                u.symbol()
            };
            vec![(what, expand(u.source()))]
        }
        Source::Value { entity, prop } => {
            let src = entity.value(*prop).map_or(entity.source(), |v| v.source);
            vec![(
                format!("{} · {}", entity.display(), prop.name()),
                src.to_string(),
            )]
        }
        Source::Entity(e) => {
            // The entity's source, then any value or fact with its own.
            let mut rows = vec![(e.display().to_string(), e.source().to_string())];
            for v in e.values().filter(|v| v.source != e.source()) {
                rows.push((
                    format!("{} · {}", e.display(), v.prop.name()),
                    v.source.to_string(),
                ));
            }
            for f in e.facts().filter(|f| f.source != e.source()) {
                rows.push((
                    format!("{} · {}", e.display(), f.name),
                    f.source.to_string(),
                ));
            }
            rows
        }
    }
}

pub fn pod(sources: &[Source]) -> Option<Pod> {
    let rows: Vec<(String, String)> = sources.iter().flat_map(row).collect();
    if rows.is_empty() {
        return None;
    }
    let width = rows
        .iter()
        .map(|(w, _)| w.chars().count())
        .max()
        .unwrap_or(0);
    let lines = rows
        .into_iter()
        .map(|(what, src)| Line(vec![Seg::Text(format!("{what:<width$}  ")), Seg::Dim(src)]))
        .collect();
    Some(Pod {
        title: "sources".into(),
        error: false,
        body: Body::Lines(lines),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_expand() {
        assert_eq!(
            expand("HB44 C-4"),
            "NIST Handbook 44 (2024), Appendix C, C-4"
        );
        assert_eq!(expand("CGPM22"), "27th CGPM (2022), Resolution 3");
        assert_eq!(expand("CODATA 2022 (NIST)"), "CODATA 2022 (NIST)");
    }
}
