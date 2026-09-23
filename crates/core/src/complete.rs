//! Tab completion: units, things, properties, and a session's names. After a
//! possessive (`earth's <tab>`) only that thing's properties are offered.

use crate::resolve::{FUNCTIONS, Scope};
use std::collections::BTreeSet;
use std::ops::Range;

/// What to offer for the word being typed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Completion {
    /// The part of the line a candidate replaces.
    pub span: Range<usize>,
    pub candidates: Vec<Candidate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
    /// What it is: `kilometre`, `planet`, `variable`.
    pub note: String,
}

/// At most this many candidates; a longer list is noise.
const MAX: usize = 40;

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Candidates for the word ending at byte `pos` of `line`.
pub fn complete(line: &str, pos: usize, scope: &Scope) -> Completion {
    let pos = pos.min(line.len());
    let before = &line[..pos];
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(pos, |(i, _)| i);
    let prefix = &line[start..pos];
    let head = &line[..start];

    let candidates = match owner(head) {
        Some(entity) => properties(entity, prefix),
        None if prefix.is_empty() || prefix.starts_with(|c: char| c.is_ascii_digit()) => Vec::new(),
        None => words(prefix, scope),
    };
    Completion {
        span: start..pos,
        candidates,
    }
}

/// `earth's ` or `earth.`: the thing whose properties come next.
fn owner(head: &str) -> Option<tungsten_kb::Entity> {
    let rest = head.trim_end();
    let rest = rest
        .strip_suffix("'s")
        .or_else(|| rest.strip_suffix("’s"))
        .or_else(|| head.strip_suffix('.'))?;
    // The longest name that ends here: `olympic pool's`, then `pool's`.
    let words: Vec<&str> = rest.split_whitespace().collect();
    (1..=words.len().min(4)).rev().find_map(|n| {
        let name = words[words.len() - n..].join(" ");
        tungsten_kb::lookup(&name)
            .into_iter()
            .find(|h| !h.shadowed)
            .map(|h| h.entity)
    })
}

fn properties(e: tungsten_kb::Entity, prefix: &str) -> Vec<Candidate> {
    let lower = prefix.to_lowercase();
    let mut seen = BTreeSet::new();
    let mut out: Vec<Candidate> = e
        .values()
        .map(|v| v.prop.name().to_string())
        .chain(e.facts().map(|f| f.name.to_string()))
        .filter(|n| n.to_lowercase().starts_with(&lower) && seen.insert(n.clone()))
        .map(|text| Candidate {
            text,
            note: e.display().to_string(),
        })
        .collect();
    out.sort_by(|a, b| a.text.cmp(&b.text));
    out.truncate(MAX);
    out
}

fn words(prefix: &str, scope: &Scope) -> Vec<Candidate> {
    let lower = prefix.to_lowercase();
    // Symbols match case exactly (`Pa` is not `pa`); names ignore case.
    let names = |s: &str| s.to_lowercase().starts_with(&lower);
    let mut out: Vec<(u8, Candidate)> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut add = |rank: u8, text: &str, note: String| {
        if seen.insert(text.to_string()) {
            out.push((
                rank,
                Candidate {
                    text: text.to_string(),
                    note,
                },
            ));
        }
    };

    for v in scope.vars.iter().filter(|v| v.starts_with(prefix)) {
        add(0, v, "variable".into());
    }
    for f in scope.funcs.iter().filter(|f| f.starts_with(prefix)) {
        add(0, f, "function".into());
    }
    for f in FUNCTIONS.iter().filter(|f| f.starts_with(&lower)) {
        add(1, f, "function".into());
    }
    for c in ["pi", "tau"].iter().filter(|c| c.starts_with(&lower)) {
        add(1, c, "constant".into());
    }
    for form in tungsten_units::all_forms() {
        let symbol = tungsten_units::lookup_symbol(form).is_some();
        let hit = if symbol {
            form.starts_with(prefix)
        } else {
            names(form)
        };
        if hit && let Some(u) = tungsten_units::lookup(form) {
            add(2, form, u.name(false));
        }
    }
    for name in tungsten_kb::all_names().filter(|n| names(n)) {
        if let Some(h) = tungsten_kb::lookup(name).into_iter().find(|h| !h.shadowed) {
            add(3, name, h.entity.kind().name().to_string());
        } else if !tungsten_kb::lookup_prop(name).is_empty() {
            add(4, name, "property".into());
        }
    }

    // Closest first: a session's names, then exact-case matches, then short.
    out.sort_by(|(ra, a), (rb, b)| {
        let case = |c: &Candidate| !c.text.starts_with(prefix);
        (case(a), *ra, a.text.len(), &a.text).cmp(&(case(b), *rb, b.text.len(), &b.text))
    });
    out.into_iter().take(MAX).map(|(_, c)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(line: &str, scope: &Scope) -> Vec<String> {
        complete(line, line.len(), scope)
            .candidates
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    #[test]
    fn after_a_possessive_only_its_properties() {
        let none = Scope::default();
        let all = texts("earth's ", &none);
        assert!(all.contains(&"mass".to_string()), "{all:?}");
        assert!(all.contains(&"radius".to_string()), "{all:?}");
        let earth = tungsten_kb::lookup("earth")[0].entity;
        for p in &all {
            assert!(
                earth.values().any(|v| v.prop.name() == p) || earth.fact(p).is_some(),
                "{p} is not one of earth's"
            );
        }
        assert_eq!(texts("earth's ma", &none), ["mass"]);
        assert_eq!(texts("earth.ma", &none), ["mass"]);
        let c = complete("earth's ma", 10, &none);
        assert_eq!(c.span, 8..10);
    }

    #[test]
    fn units_and_things() {
        let none = Scope::default();
        let k = texts("kilom", &none);
        assert!(k.contains(&"kilometre".to_string()), "{k:?}");
        let j = texts("jupi", &none);
        assert_eq!(j.first().map(String::as_str), Some("jupiter"), "{j:?}");
        assert!(texts("", &none).is_empty());
        assert!(texts("5", &none).is_empty());
    }

    #[test]
    fn session_names_first() {
        let scope = Scope {
            vars: ["rent".to_string()].into(),
            funcs: BTreeSet::new(),
        };
        let r = texts("re", &scope);
        assert_eq!(r[0], "rent");
        assert_eq!(complete("2 * re", 6, &scope).span, 4..6);
    }
}
