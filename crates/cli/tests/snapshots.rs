//! Golden-output tests: every query in tests/queries.txt, rendered at widths
//! 80 and 40, compared against tests/snap/. The footer's timing is off, so
//! snapshots are stable.
//!
//! Also keeps the README's example blocks identical to their snapshots.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tungsten::{Settings, render_query};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn queries() -> Vec<String> {
    std::fs::read_to_string(root().join("tests/queries.txt"))
        .expect("tests/queries.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

fn slug(q: &str) -> String {
    let mut s = String::new();
    for c in q.chars() {
        let mapped = match c {
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase().to_string(),
            '+' => "p".into(),
            '%' => "c".into(),
            '!' => "f".into(),
            '°' => "d".into(),
            // Other non-ASCII keeps its identity, so ½ and Δ never collide.
            c if !c.is_ascii() => format!("u{:x}", c as u32),
            _ => "-".into(),
        };
        if mapped == "-" && (s.ends_with('-') || s.is_empty()) {
            continue;
        }
        s.push_str(&mapped);
    }
    s.trim_end_matches('-').chars().take(60).collect()
}

/// Corpus lines may start with flags: `--why earth.mass`, `--as element W`.
fn snapshot(line: &str, width: usize) -> String {
    let mut s = Settings {
        width,
        color: false,
        fancy: true,
        sig: None,
        timing: false,
        prefer: None,
        why: false,
    };
    let mut q = line;
    loop {
        if let Some(rest) = q.strip_prefix("--why ") {
            s.why = true;
            q = rest;
        } else if let Some(rest) = q.strip_prefix("--as ") {
            let (kind, rest) = rest.split_once(' ').expect("--as KIND query");
            s.prefer = Some(tungsten_core::Kind::parse(kind).expect("known kind"));
            q = rest;
        } else {
            break;
        }
    }
    if q == "--about" {
        return format!("$ w {line}\n{}", tungsten::render_about(&s).out.trim_end());
    }
    format!("$ w {line}\n{}", render_query(q, &s).out.trim_end())
}

#[test]
fn corpus() {
    let qs = queries();
    let mut slugs = HashSet::new();
    for q in &qs {
        assert!(
            slugs.insert(slug(q)),
            "two queries share the slug {}",
            slug(q)
        );
    }
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(root().join("tests/snap"));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings.bind(|| {
        for q in &qs {
            for width in [80, 40] {
                insta::assert_snapshot!(format!("{}@{width}", slug(q)), snapshot(q, width));
            }
        }
    });
}

/// README blocks between `<!-- snap: SLUG -->` and `<!-- /snap -->` must equal
/// the 80-column snapshot of that query. `UPDATE_README=1` rewrites them.
#[test]
fn readme_matches_snapshots() {
    let path = root().join("README.md");
    let Ok(readme) = std::fs::read_to_string(&path) else {
        return;
    };
    let by_slug: HashMap<String, String> = queries().iter().map(|q| (slug(q), q.clone())).collect();
    let mut out = String::new();
    let mut rest = readme.as_str();
    while let Some(start) = rest.find("<!-- snap: ") {
        let open_end = start + rest[start..].find("-->").expect("unclosed marker") + 3;
        let name = rest[start + 11..open_end - 3].trim();
        let close = open_end
            + rest[open_end..]
                .find("<!-- /snap -->")
                .expect("missing /snap");
        let q = by_slug
            .get(name)
            .unwrap_or_else(|| panic!("README names unknown query {name}"));
        out.push_str(&rest[..open_end]);
        out.push_str(&format!("\n```text\n{}\n```\n", snapshot(q, 80)));
        rest = &rest[close..];
    }
    out.push_str(rest);
    if std::env::var_os("UPDATE_README").is_some() {
        std::fs::write(&path, &out).expect("write README");
    } else {
        assert!(
            out == readme,
            "README examples are stale; run UPDATE_README=1 cargo test"
        );
    }
}
