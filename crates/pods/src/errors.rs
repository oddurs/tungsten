//! Error pods (docs/concept.md §7).
//!
//! Every error shows the query and points at the part that is wrong, either
//! with carets or, for dimension mismatches, with a bracket under each operand
//! naming its dimension.

use crate::{Body, Line, NumMode, Pod, Seg};
use std::ops::Range;
use tungsten_core::{Error, ErrorKind, Hint};
use tungsten_units::{Dim, MathError, describe};
use unicode_width::UnicodeWidthStr;

fn col(input: &str, byte: usize) -> usize {
    input.get(..byte).map_or(0, UnicodeWidthStr::width)
}

fn width(input: &str, span: &Range<usize>) -> usize {
    input
        .get(span.clone())
        .map_or(1, UnicodeWidthStr::width)
        .max(1)
}

fn caret(input: &str, span: &Range<usize>, note: &str) -> Line {
    let mut l = Line::default();
    l.push(Seg::Text(" ".repeat(col(input, span.start))));
    l.push(Seg::Error("^".repeat(width(input, span))));
    if !note.is_empty() {
        l.push(Seg::Dim(format!(" {note}")));
    }
    l
}

fn text(s: impl Into<String>) -> Line {
    Line(vec![Seg::Text(s.into())])
}

fn dim(s: impl Into<String>) -> Line {
    Line(vec![Seg::Dim(s.into())])
}

/// `a length`, `an area`: describe a dimension with its article.
fn a(d: &Dim) -> String {
    let name = describe(d, true);
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {name}")
}

fn hint_line(h: &Hint) -> Line {
    let mut l = Line::default();
    l.push(Seg::Dim("hint: did you mean ".into()));
    l.push(Seg::Text(h.text.clone()));
    l.push(Seg::Dim("  (= ".into()));
    if let Ok(num) = h.value.shown() {
        l.push(Seg::Value {
            num,
            unit: h.value.unit.clone(),
            mode: NumMode::Result,
        });
    }
    l.push(Seg::Dim(")?".into()));
    l
}

/// Brackets under each operand, labelled right to left:
///
/// ```text
/// 3 m + 2 s
/// ─┬─   ─┬─
///  │     └ time
///  └ length
/// ```
fn diagram(input: &str, operands: &[(Range<usize>, Dim)]) -> Vec<Line> {
    let marks: Vec<(usize, usize, String)> = operands
        .iter()
        .map(|(span, d)| {
            let start = col(input, span.start);
            let w = width(input, span);
            (start, start + (w - 1) / 2, describe(d, true))
        })
        .collect();
    let mut lines = Vec::new();
    let mut under = String::new();
    for ((span, _), (start, mid, _)) in operands.iter().zip(&marks) {
        let w = width(input, span);
        under.push_str(&" ".repeat(start.saturating_sub(under.chars().count())));
        for i in 0..w {
            under.push(if start + i == *mid { '┬' } else { '─' });
        }
    }
    lines.push(dim(under));
    for k in (0..marks.len()).rev() {
        let mut row = String::new();
        for (j, (_, mid, label)) in marks.iter().enumerate().take(k + 1) {
            row.push_str(&" ".repeat(mid.saturating_sub(row.chars().count())));
            if j == k {
                row.push_str("└ ");
                row.push_str(label);
            } else {
                row.push('│');
            }
        }
        lines.push(dim(row));
    }
    lines
}

/// The error pod, and a one-line summary for `-q`.
pub fn pod(input: &str, e: &Error) -> (Pod, String) {
    let shown = text(input.trim_end());
    let mut lines = vec![shown];
    let (title, summary): (String, String) = match &*e.kind {
        ErrorKind::UnexpectedChar(c) => {
            lines.push(caret(input, &e.span, "tungsten can't read this character"));
            (
                "unexpected character".into(),
                format!("unexpected character {c:?}"),
            )
        }
        ErrorKind::NumberTooLarge => {
            lines.push(caret(input, &e.span, "too large to work with"));
            ("number too large".into(), "number too large".into())
        }
        ErrorKind::UnknownWord { word, suggestion } => {
            let note = match suggestion {
                Some(s) => format!("did you mean {s}?"),
                None => "not a unit or word tungsten knows".into(),
            };
            lines.push(caret(input, &e.span, &note));
            let summary = match suggestion {
                Some(s) => format!("unknown word {word:?} (did you mean {s}?)"),
                None => format!("unknown word {word:?}"),
            };
            ("unknown word".into(), summary)
        }
        ErrorKind::Unexpected { found, expected } => {
            lines.push(caret(input, &e.span, &format!("expected {expected}")));
            (
                "can't read this".into(),
                format!("expected {expected}, found {found:?}"),
            )
        }
        ErrorKind::UnexpectedEnd { expected } => {
            let end = input.trim_end().len();
            lines.push(caret(input, &(end..end), &format!("expected {expected}")));
            (
                "incomplete query".into(),
                format!("incomplete query: expected {expected}"),
            )
        }
        ErrorKind::Empty => {
            lines = vec![dim("try: w 5 mi in km")];
            ("nothing to calculate".into(), "nothing to calculate".into())
        }
        ErrorKind::Mismatch { op, operands, hint } => {
            lines.extend(diagram(input, operands));
            if let Some(h) = hint {
                lines.push(hint_line(h));
            }
            let verb = if *op == '+' { "add" } else { "subtract" };
            let names: Vec<String> = operands.iter().map(|(_, d)| describe(d, true)).collect();
            (
                format!("can't {verb} these"),
                format!("can't {verb} {}", names.join(" and ")),
            )
        }
        ErrorKind::AddTemperatures { hint } => {
            lines.push(dim(
                "both are readings on a temperature scale; add a difference instead",
            ));
            if let Some(h) = hint {
                lines.push(hint_line(h));
            }
            (
                "can't add two temperatures".into(),
                "can't add two temperatures".into(),
            )
        }
        ErrorKind::NegateTemperature => {
            lines.push(caret(input, &e.span, "put the sign on the number: -40 °C"));
            (
                "can't negate a temperature".into(),
                "can't negate a temperature reading".into(),
            )
        }
        ErrorKind::Convert { from, to } => {
            lines.push(caret(input, &e.span, ""));
            let target = input.get(e.span.clone()).unwrap_or("that unit").trim();
            let mut l = Line::default();
            l.push(Seg::Dim("the query is ".into()));
            l.push(Seg::Text(a(from)));
            l.push(Seg::Dim(format!(", but {target} measures ")));
            l.push(Seg::Text(describe(to, true)));
            lines.push(l);
            (
                "can't convert".into(),
                format!(
                    "can't convert {} to {}",
                    describe(from, true),
                    describe(to, true)
                ),
            )
        }
        ErrorKind::NeedsNumber { what, dim: d } => {
            lines.push(caret(
                input,
                &e.span,
                &format!("{what} needs a plain number, not {}", a(d)),
            ));
            (
                "needs a plain number".into(),
                format!("{what} needs a plain number"),
            )
        }
        ErrorKind::IrrationalPower { dim: d } => {
            lines.push(caret(
                input,
                &e.span,
                &format!("{} can only be raised to a fraction", a(d)),
            ));
            (
                "can't take this power".into(),
                "irrational power of a quantity".into(),
            )
        }
        ErrorKind::WrongArity {
            func,
            expected,
            found,
        } => {
            let note = format!("{func} takes {expected} argument, not {found}");
            lines.push(caret(input, &e.span, &note));
            ("wrong number of arguments".into(), note)
        }
        ErrorKind::FactorialDomain => {
            lines.push(caret(input, &e.span, "factorials need a whole number"));
            (
                "can't take this factorial".into(),
                "factorial of a non-integer".into(),
            )
        }
        ErrorKind::NoProperty { entity, prop } => {
            let has: Vec<&str> = entity.values().map(|v| v.prop.name()).take(6).collect();
            let note = if has.is_empty() {
                format!("{} has no {prop}", entity.display())
            } else {
                format!("{} has no {prop}; try {}", entity.display(), has.join(", "))
            };
            lines.push(caret(input, &e.span, &note));
            (
                "no such property".into(),
                format!("{} has no {prop}", entity.display()),
            )
        }
        ErrorKind::NoValue { entity } => {
            let example = entity
                .values()
                .next()
                .map(|v| {
                    format!(
                        "; ask for one of its properties, like {} of {}",
                        v.prop.name(),
                        entity.display()
                    )
                })
                .unwrap_or_default();
            let note = format!("{} is a thing, not a quantity{example}", entity.display());
            lines.push(caret(input, &e.span, &note));
            (
                "not a quantity".into(),
                format!("{} is not a quantity", entity.display()),
            )
        }
        ErrorKind::NoIt => {
            lines.push(caret(input, &e.span, "nothing has been calculated yet"));
            (
                "no previous answer".into(),
                "no previous answer for it".into(),
            )
        }
        ErrorKind::CannotAssign {
            name,
            reason,
            instead,
        } => {
            lines.push(caret(input, &e.span, reason));
            if let Some(other) = instead {
                let fixed = input.trim().replacen(name.as_str(), other, 1);
                lines.push(dim(format!("try {fixed}")));
            }
            (
                format!("can't define {name}"),
                format!("can't define {name}: {reason}"),
            )
        }
        ErrorKind::UserArity {
            name,
            expected,
            found,
        } => {
            let s = if *expected == 1 { "" } else { "s" };
            let note = format!("{name} takes {expected} argument{s}, not {found}");
            lines.push(caret(input, &e.span, &note));
            ("wrong number of arguments".into(), note)
        }
        ErrorKind::TooDeep { name } => {
            let note = format!("{name} calls functions more than 16 deep");
            lines.push(caret(input, &e.span, &note));
            ("too deep".into(), note)
        }
        ErrorKind::NotAThing => {
            lines.push(caret(input, &e.span, "means something else here; see --as"));
            ("not a thing here".into(), "not a thing here".into())
        }
        ErrorKind::Math(m) => {
            let (title, note) = match m {
                MathError::DivideByZero => ("can't divide by zero", "this is zero"),
                MathError::NotReal => ("no real answer", "the answer is not a real number"),
                MathError::Overflow => ("too large", "the answer is too large to represent"),
            };
            lines.push(caret(input, &e.span, note));
            (title.into(), title.into())
        }
    };
    (
        Pod {
            title,
            error: true,
            body: Body::Pre(lines),
        },
        summary,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(l: &Line) -> String {
        l.0.iter()
            .map(|s| match s {
                Seg::Text(t) | Seg::Dim(t) | Seg::Error(t) => t.clone(),
                _ => "<v>".into(),
            })
            .collect()
    }

    #[test]
    fn mismatch_diagram_matches_the_spec() {
        let input = "3 m + 2 s";
        let e = tungsten_core::evaluate(input).unwrap_err();
        let (pod, summary) = pod(input, &e);
        let Body::Pre(lines) = pod.body else { panic!() };
        let got: Vec<String> = lines.iter().map(plain).collect();
        assert_eq!(got[0], "3 m + 2 s");
        assert_eq!(got[1], "─┬─   ─┬─");
        assert_eq!(got[2], " │     └ time");
        assert_eq!(got[3], " └ length");
        assert!(got[4].starts_with("hint: did you mean 3 m / 2 s  (= "));
        assert_eq!(summary, "can't add length and time");
    }

    #[test]
    fn unknown_word_caret() {
        let input = "5 kilometers in milez";
        let e = tungsten_core::evaluate(input).unwrap_err();
        let (pod, _) = pod(input, &e);
        let Body::Pre(lines) = pod.body else { panic!() };
        assert_eq!(
            plain(&lines[1]),
            "                ^^^^^ did you mean miles?"
        );
    }
}
