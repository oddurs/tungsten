//! Compiles data/{properties,constants,elements,solar,items}.toml into static
//! tables, validating everything first so bad data fails the build rather
//! than a query:
//!
//! - every value parses with the real unit parser, and has its property's
//!   dimension;
//! - every property is declared in properties.toml;
//! - every entity has a source; defaults, scale lists and per-value sources
//!   name properties the entity has;
//! - a name or symbol that is already a unit, keyword or constant is kept but
//!   *shadowed* (reachable only with `--as`), for constants and elements; for
//!   hand-written kinds it is an error.
//!
//! Errors name the file and line.

use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use toml::Spanned;

const FILES: &[&str] = &["constants", "elements", "solar", "food", "items"];

const KINDS: &[(&str, &str)] = &[
    ("constant", "Constant"),
    ("element", "Element"),
    ("star", "Star"),
    ("planet", "Planet"),
    ("dwarf planet", "DwarfPlanet"),
    ("moon", "Moon"),
    ("item", "Item"),
];

/// Words the query language already owns (crates/core/src/resolve.rs).
#[rustfmt::skip]
const RESERVED: &[&str] = &[
    "in", "to", "as", "into", "per", "a", "an", "of", "for", "times", "x", "plus", "minus",
    "over", "squared", "cubed", "square", "sq", "cubic", "cu", "half", "twice", "double", "and",
    "is", "are", "there", "many", "much", "how", "the", "pi", "π", "tau", "τ", "e", "sqrt",
    "cbrt", "sin", "cos", "tan", "asin", "acos", "atan", "arcsin", "arccos", "arctan", "ln",
    "log", "log10", "log2", "exp", "abs", "round", "floor", "ceil", "hundred", "thousand",
    "million", "billion", "trillion", "quadrillion", "dozen", "percent", "pct", "permille",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropFile {
    property: Vec<Property>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Property {
    names: Vec<String>,
    #[serde(default)]
    also: Vec<String>,
    dim: Option<String>,
    show: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntityFile {
    #[serde(default)]
    entity: Vec<Spanned<Entity>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entity {
    kind: String,
    display: String,
    names: Vec<String>,
    #[serde(default)]
    symbols: Vec<String>,
    plural: Option<String>,
    default: Option<String>,
    #[serde(default)]
    scale: Vec<String>,
    domain: Option<String>,
    source: String,
    #[serde(default)]
    props: IndexMap<String, Spanned<String>>,
    #[serde(default)]
    uncertainty: IndexMap<String, String>,
    #[serde(default)]
    facts: IndexMap<String, String>,
    #[serde(default)]
    sources: IndexMap<String, String>,
}

struct Ctx {
    file: String,
    text: String,
}

impl Ctx {
    fn line(&self, byte: usize) -> usize {
        self.text[..byte.min(self.text.len())].matches('\n').count() + 1
    }

    fn fail(&self, byte: usize, msg: impl std::fmt::Display) -> ! {
        panic!("\n\n{}:{}: {msg}\n\n", self.file, self.line(byte));
    }
}

fn read(dir: &Path, name: &str) -> Ctx {
    let path = dir.join(format!("{name}.toml"));
    println!("cargo:rerun-if-changed={}", path.display());
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    Ctx {
        file: format!("data/{name}.toml"),
        text,
    }
}

fn pluralize(s: &str) -> String {
    let (head, last) = match s.rsplit_once(' ') {
        Some((h, l)) => (format!("{h} "), l.to_string()),
        None => (String::new(), s.to_string()),
    };
    let b = last.as_bytes();
    let n = b.len();
    let p = if last.ends_with('s')
        || last.ends_with('x')
        || last.ends_with("ch")
        || last.ends_with("sh")
    {
        format!("{last}es")
    } else if n > 1 && b[n - 1] == b'y' && !b"aeiou".contains(&b[n - 2]) {
        format!("{}ies", &last[..n - 1])
    } else {
        format!("{last}s")
    };
    format!("{head}{p}")
}

/// Does this word already mean something to the query language?
fn taken(word: &str) -> Option<String> {
    if RESERVED.contains(&word.to_lowercase().as_str()) {
        return Some("a keyword".into());
    }
    if let Some(u) = tungsten_units::lookup_symbol(word) {
        return Some(format!("the unit {}", u.symbol()));
    }
    if let Some(u) = tungsten_units::lookup_name(word) {
        return Some(format!("the unit {}", u.symbol()));
    }
    None
}

fn main() {
    let dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest")).join("../../data");

    // ── Properties ──────────────────────────────────────────────────────────
    let pctx = read(&dir, "properties");
    let props: PropFile =
        toml::from_str(&pctx.text).unwrap_or_else(|e| panic!("\n\ndata/properties.toml: {e}\n\n"));
    let mut prop_index: BTreeMap<String, usize> = BTreeMap::new();
    let mut prop_names: Vec<(String, usize, bool)> = Vec::new();
    let mut prop_dims = Vec::new();
    for (i, p) in props.property.iter().enumerate() {
        prop_index.insert(p.names[0].clone(), i);
        for n in &p.names {
            assert_eq!(
                n,
                &n.to_lowercase(),
                "data/properties.toml: {n:?} must be lowercase"
            );
            prop_names.push((n.clone(), i, false));
        }
        for n in &p.also {
            prop_names.push((n.clone(), i, true));
        }
        let dim = p.dim.as_ref().map(|d| {
            tungsten_units::UnitExpr::parse(d)
                .unwrap_or_else(|| panic!("data/properties.toml: bad dim {d:?} for {}", p.names[0]))
                .dim()
        });
        if let Some(s) = &p.show {
            let u = tungsten_units::UnitExpr::parse(s).unwrap_or_else(|| {
                panic!(
                    "data/properties.toml: bad show unit {s:?} for {}",
                    p.names[0]
                )
            });
            if let Some(d) = dim {
                assert_eq!(
                    u.dim(),
                    d,
                    "data/properties.toml: show unit {s} has the wrong dimension"
                );
            }
        }
        prop_dims.push(dim);
    }
    prop_names.sort();
    for w in prop_names.windows(2) {
        if w[0].0 == w[1].0 && !(w[0].2 && w[1].2) {
            panic!("data/properties.toml: {:?} names two properties", w[0].0);
        }
    }

    let mut out = String::from("// Generated by build.rs from data/*.toml. Do not edit.\n\n");
    out.push_str("pub static PROPS: &[PropDef] = &[\n");
    for p in &props.property {
        writeln!(
            out,
            "    PropDef {{ names: &{:?}, show: {:?} }},",
            p.names, p.show
        )
        .ok();
    }
    out.push_str("];\n\n");
    out.push_str("pub static PROP_NAMES: &[(&str, u16, bool)] = &[\n");
    for (n, i, also) in &prop_names {
        writeln!(out, "    ({n:?}, {i}, {also}),").ok();
    }
    out.push_str("];\n\n");

    // ── Entities ────────────────────────────────────────────────────────────
    let mut names: Vec<(String, usize, bool)> = Vec::new();
    let mut symbols: Vec<(String, usize, bool)> = Vec::new();
    let mut max_words = 1;
    let mut count = 0usize;
    out.push_str("pub static ENTITIES: &[EntityDef] = &[\n");
    for f in FILES {
        let ctx = read(&dir, f);
        let file: EntityFile =
            toml::from_str(&ctx.text).unwrap_or_else(|e| panic!("\n\ndata/{f}.toml: {e}\n\n"));
        for spanned in &file.entity {
            let at = spanned.span().start;
            let e = spanned.get_ref();
            let kind = KINDS
                .iter()
                .find(|(k, _)| *k == e.kind)
                .unwrap_or_else(|| ctx.fail(at, format!("unknown kind {:?}", e.kind)))
                .1;
            if e.source.trim().is_empty() {
                ctx.fail(at, format!("{} has no source", e.display));
            }
            if e.names.is_empty() {
                ctx.fail(at, format!("{} has no names", e.display));
            }
            let lenient = matches!(e.kind.as_str(), "constant" | "element");
            let idx = count;
            count += 1;

            for n in &e.names {
                if *n != n.to_lowercase() {
                    ctx.fail(at, format!("name {n:?} must be lowercase"));
                }
                if prop_names.iter().any(|(p, ..)| p == n) {
                    ctx.fail(at, format!("name {n:?} is also a property"));
                }
                let shadow = taken(n);
                if shadow.is_some() && !lenient {
                    ctx.fail(
                        at,
                        format!("name {n:?} is already {}", shadow.unwrap_or_default()),
                    );
                }
                max_words = max_words.max(n.split(' ').count());
                names.push((n.clone(), idx, shadow.is_some()));
            }
            for s in &e.symbols {
                let shadow = taken(s);
                if shadow.is_some() && !lenient {
                    ctx.fail(
                        at,
                        format!("symbol {s:?} is already {}", shadow.unwrap_or_default()),
                    );
                }
                symbols.push((s.clone(), idx, shadow.is_some()));
            }

            let mut prop_rows = Vec::new();
            for (key, value) in &e.props {
                let vat = value.span().start;
                let Some(&pi) = prop_index.get(key) else {
                    ctx.fail(
                        vat,
                        format!("{key:?} is not declared in data/properties.toml"),
                    )
                };
                let v = value.get_ref();
                let parsed = tungsten_units::parse_def(v).unwrap_or_else(|err| {
                    ctx.fail(vat, format!("{} {key} = {v:?}: {err}", e.display))
                });
                if let Some(d) = prop_dims[pi] {
                    let got = tungsten_units::UnitExpr::from_terms(parsed.terms.clone()).dim();
                    if got != d {
                        ctx.fail(
                            vat,
                            format!(
                                "{} {key} = {v:?} is {}, but {key} is {}",
                                e.display,
                                tungsten_units::describe(&got, false),
                                tungsten_units::describe(&d, false)
                            ),
                        );
                    }
                }
                prop_rows.push(format!(
                    "({pi}, {v:?}, {:?}, {:?})",
                    e.uncertainty.get(key),
                    e.sources.get(key)
                ));
            }
            let has = |p: &str| e.props.contains_key(p);
            for k in e.uncertainty.keys() {
                if !has(k) {
                    ctx.fail(
                        at,
                        format!("uncertainty for {k:?}, which {} does not have", e.display),
                    );
                }
            }
            for k in e.sources.keys() {
                if !has(k) && !e.facts.contains_key(k) {
                    ctx.fail(
                        at,
                        format!("source for {k:?}, which {} does not have", e.display),
                    );
                }
            }
            let default = e.default.as_ref().map(|d| {
                if !has(d) {
                    ctx.fail(
                        at,
                        format!("default {d:?} is not a property of {}", e.display),
                    );
                }
                prop_index[d]
            });
            let scale: Vec<usize> = e
                .scale
                .iter()
                .map(|s| {
                    if !has(s) {
                        ctx.fail(
                            at,
                            format!("scale {s:?} is not a property of {}", e.display),
                        );
                    }
                    prop_index[s]
                })
                .collect();
            let plural = e.plural.clone().unwrap_or_else(|| pluralize(&e.display));
            let facts: Vec<String> = e
                .facts
                .iter()
                .map(|(k, v)| format!("({k:?}, {v:?}, {:?})", e.sources.get(k)))
                .collect();
            writeln!(
                out,
                "    EntityDef {{ kind: Kind::{kind}, display: {:?}, plural: {plural:?}, names: &{:?}, \
                 symbols: &{:?}, default: {default:?}, scale: &{scale:?}, domain: {:?}, source: {:?}, \
                 props: &[{}], facts: &[{}] }},",
                e.display,
                e.names,
                e.symbols,
                e.domain,
                e.source,
                prop_rows.join(", "),
                facts.join(", ")
            )
            .ok();
        }
    }
    out.push_str("];\n\n");

    names.sort();
    symbols.sort();
    for (label, list) in [("NAMES", &names), ("SYMBOLS", &symbols)] {
        writeln!(out, "pub static {label}: &[(&str, u16, bool)] = &[").ok();
        for (n, i, shadow) in list {
            writeln!(out, "    ({n:?}, {i}, {shadow}),").ok();
        }
        out.push_str("];\n\n");
    }
    writeln!(out, "pub const MAX_NAME_WORDS: usize = {max_words};").ok();

    let dest = PathBuf::from(std::env::var("OUT_DIR").expect("out")).join("kb.rs");
    std::fs::write(dest, out).expect("write kb.rs");
}
