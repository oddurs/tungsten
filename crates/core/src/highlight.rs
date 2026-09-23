//! What each part of a line means, for highlighting it as it is typed. Uses
//! the resolver evaluation uses, so what is coloured is what will be read.

use crate::lex;
use crate::resolve::{self, Scope, Sym};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// `5`, `million`, `pi`.
    Number,
    /// `km`, `miles per hour`, `'`.
    Unit,
    /// `earth`, `olympic pool`.
    Entity,
    /// `mass`, `boiling point`.
    Property,
    /// A session's variables and functions, and `it`.
    Variable,
    /// `sqrt`, `sin`.
    Function,
    /// `in`, `per`, `how many`.
    Keyword,
    /// A word tungsten does not know, or a character it cannot read.
    Unknown,
}

/// Spans of `src` and what they are, in order. Operators, spaces and filler
/// words (`what is`) have no class.
pub fn classify(src: &str, scope: &Scope) -> Vec<(Range<usize>, Class)> {
    let tokens = match lex::lex(src) {
        Ok(t) => t,
        Err(e) => {
            // Everything before the unreadable character still counts.
            let start = e.span.start.min(src.len());
            let mut out = classify(&src[..start], scope);
            let end = src[start..]
                .chars()
                .next()
                .map_or(start, |c| start + c.len_utf8());
            if end > start {
                out.push((start..end, Class::Unknown));
            }
            return out;
        }
    };
    let mut unknown = Vec::new();
    let items = resolve::resolve_lenient(&tokens, scope, &mut unknown);
    let mut out: Vec<(Range<usize>, Class)> = items
        .into_iter()
        .filter_map(|it| {
            let class = match it.sym {
                Sym::Num { .. } | Sym::Magnitude(_) | Sym::Const(_) => Class::Number,
                Sym::Unit(_) | Sym::UnitPow(..) | Sym::Foot | Sym::Inch => Class::Unit,
                Sym::Name(m) => {
                    if m.usable().next().is_some() {
                        Class::Entity
                    } else if !m.props.is_empty() {
                        Class::Property
                    } else if m.unit.is_some() {
                        Class::Unit
                    } else {
                        Class::Entity
                    }
                }
                Sym::Var(_) | Sym::UserFunc(_) => Class::Variable,
                Sym::Func(_) => Class::Function,
                Sym::Kw(_) | Sym::Ask(_) => Class::Keyword,
                Sym::Op(_) | Sym::Possessive => return None,
            };
            Some((it.span, class))
        })
        .chain(unknown.into_iter().map(|r| (r, Class::Unknown)))
        .collect();
    out.sort_by_key(|(r, _)| r.start);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classes<'a>(src: &'a str, scope: &Scope) -> Vec<(&'a str, Class)> {
        classify(src, scope)
            .into_iter()
            .map(|(r, c)| (&src[r], c))
            .collect()
    }

    #[test]
    fn as_the_resolver_reads_it() {
        let none = Scope::default();
        assert_eq!(
            classes("60 miles per hour in km/h", &none),
            [
                ("60", Class::Number),
                ("miles per hour", Class::Unit),
                ("in", Class::Keyword),
                ("km", Class::Unit),
                ("h", Class::Unit),
            ]
        );
        assert_eq!(
            classes("mass of earth", &none),
            [
                ("mass", Class::Property),
                ("of", Class::Keyword),
                ("earth", Class::Entity),
            ]
        );
        assert_eq!(
            classes("sqrt(2) * pi", &none),
            [
                ("sqrt", Class::Function),
                ("2", Class::Number),
                ("pi", Class::Number),
            ]
        );
    }

    #[test]
    fn unknown_words_and_characters() {
        let none = Scope::default();
        assert_eq!(
            classes("5 blorps in m", &none),
            [
                ("5", Class::Number),
                ("blorps", Class::Unknown),
                ("in", Class::Keyword),
                ("m", Class::Unit),
            ]
        );
        assert_eq!(
            classes("5 m @ 3", &none),
            [
                ("5", Class::Number),
                ("m", Class::Unit),
                ("@", Class::Unknown)
            ]
        );
        // `it` means nothing until there is a previous answer.
        assert_eq!(classes("it", &none), [("it", Class::Unknown)]);
    }

    #[test]
    fn session_names() {
        let scope = Scope {
            vars: ["rent".to_string(), "a".to_string(), "it".to_string()].into(),
            funcs: ["f".to_string()].into(),
        };
        assert_eq!(
            classes("rent * a + f(it)", &scope),
            [
                ("rent", Class::Variable),
                ("a", Class::Variable),
                ("f", Class::Variable),
                ("it", Class::Variable),
            ]
        );
    }

    #[test]
    fn agrees_with_evaluation() {
        // Every line that evaluates has no unknown spans, and every line with
        // an unknown word fails to evaluate.
        let none = Scope::default();
        for q in [
            "5 mi in km",
            "how many feet in a mile",
            "earth's mass / moon's mass",
            "speed of light in mph",
            "3 cups of flour in g",
        ] {
            assert!(crate::evaluate(q).is_ok(), "{q}");
            assert!(
                classify(q, &none).iter().all(|(_, c)| *c != Class::Unknown),
                "{q}"
            );
        }
        for q in ["5 blorps", "earth's zorp"] {
            assert!(crate::evaluate(q).is_err(), "{q}");
            assert!(
                classify(q, &none).iter().any(|(_, c)| *c == Class::Unknown),
                "{q}"
            );
        }
    }
}
