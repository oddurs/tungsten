//! Turns an evaluated query (or its error) into pods.
//!
//! A pod is semantic: numbers stay [`Number`]s and units stay [`UnitExpr`]s,
//! so the render crate alone decides formatting, colour and layout.
//! See docs/concept.md §5 and §7.

mod card;
mod errors;
mod other_units;

use tungsten_core::{Answer, Error, Kind, Outcome, Piece, interpret};
use tungsten_units::{Number, UnitExpr};

pub use other_units::other_units;

/// How a number should be formatted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumMode {
    /// As the answer: exact where exact, fractions where the input was
    /// fractions, rounded to significant figures otherwise.
    Result,
    /// Rounded to significant figures (other units, decimal approximations).
    Rounded,
    /// As written in the query (interpretation).
    Literal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Seg {
    Text(String),
    /// Secondary text: hints, labels, diagram lines.
    Dim(String),
    /// Error text.
    Error(String),
    /// A number, optionally with its unit.
    Value {
        num: Number,
        unit: UnitExpr,
        mode: NumMode,
    },
    Unit(UnitExpr),
    /// A superscript exponent.
    Sup(i128),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Line(pub Vec<Seg>);

impl Line {
    pub fn push(&mut self, s: Seg) -> &mut Self {
        self.0.push(s);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    /// Ordinary lines; the renderer may wrap them.
    Lines(Vec<Line>),
    /// Alternatives: joined on one line when wide, one per line when narrow.
    List(Vec<Line>),
    /// Preformatted: never wrapped (error diagrams).
    Pre(Vec<Line>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pod {
    pub title: String,
    pub error: bool,
    pub body: Body,
}

/// The bare answer, for `-q`.
#[derive(Clone, Debug, PartialEq)]
pub enum Quiet {
    Value(Number),
    Parts(Vec<(Number, UnitExpr)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub pods: Vec<Pod>,
    pub ok: bool,
    pub quiet: Option<Quiet>,
    /// One-line error summary, for `-q` on stderr.
    pub error_line: Option<String>,
}

pub fn build(input: &str, result: &Result<Outcome, Error>) -> Report {
    match result {
        Ok(o) => success(input, o),
        Err(e) => {
            let (pod, line) = errors::pod(input, e);
            Report {
                pods: vec![pod],
                ok: false,
                quiet: None,
                error_line: Some(line),
            }
        }
    }
}

/// `a planet`, `an element`.
fn with_article(kind: Kind) -> String {
    let name = kind.name();
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {name}")
}

/// ```text
/// ◆ assuming
/// │ "mercury" is a planet  ·  use --as element for the element
/// ```
fn assuming(input: &str, o: &Outcome) -> Option<Pod> {
    let lines: Vec<Line> = o
        .assumptions
        .iter()
        .map(|a| {
            let word = input.get(a.span.clone()).unwrap_or("").trim();
            let others: Vec<String> = a
                .others
                .iter()
                .map(|e| {
                    format!(
                        "use --as {} for the {}",
                        e.kind().name().replace(' ', "-"),
                        e.kind().name()
                    )
                })
                .collect();
            Line(vec![
                Seg::Text(format!("\"{word}\" is {}", with_article(a.chosen.kind()))),
                Seg::Dim(format!("  ·  {}", others.join(", "))),
            ])
        })
        .collect();
    (!lines.is_empty()).then(|| Pod {
        title: "assuming".into(),
        error: false,
        body: Body::Lines(lines),
    })
}

fn success(input: &str, o: &Outcome) -> Report {
    let mut pods: Vec<Pod> = assuming(input, o).into_iter().collect();

    if let Answer::Card(e) = o.answer {
        pods.push(card::card(e));
        // `-q G` prints G's value: a card with a default has one number.
        let quiet = e
            .default()
            .and_then(|p| e.value(p))
            .map(|v| Quiet::Value(v.num));
        let error_line = quiet
            .is_none()
            .then(|| format!("{} is a thing, not a quantity", e.display()));
        return Report {
            pods,
            ok: true,
            quiet,
            error_line,
        };
    }

    let mut interp = Line::default();
    for p in interpret(&o.query) {
        interp.push(match p {
            Piece::Num(num) => Seg::Value {
                num,
                unit: UnitExpr::default(),
                mode: NumMode::Literal,
            },
            Piece::Unit(u) => Seg::Unit(u),
            Piece::Op(s) => Seg::Text(s.into()),
            Piece::Sup(n) => Seg::Sup(n),
            Piece::Word(w) => Seg::Text(w.into()),
            Piece::Text(t) => Seg::Text(t),
            Piece::Space => Seg::Text(" ".into()),
            Piece::Open => Seg::Text("(".into()),
            Piece::Close => Seg::Text(")".into()),
            // No-break spaces keep the arrow with its target when wrapping.
            Piece::Arrow => Seg::Text("  →\u{a0}\u{a0}".into()),
            Piece::Comma => Seg::Text(", ".into()),
        });
    }
    pods.push(Pod {
        title: "interpretation".into(),
        error: false,
        body: Body::Lines(vec![interp]),
    });

    let (result_line, quiet) = match &o.answer {
        Answer::Single { num, unit, .. } => {
            let seg = Seg::Value {
                num: *num,
                unit: unit.clone(),
                mode: NumMode::Result,
            };
            (Line(vec![seg]), Quiet::Value(*num))
        }
        Answer::Card(_) => unreachable!("cards return early"),
        Answer::Parts(parts) => {
            let mut line = Line::default();
            for (i, (n, u)) in parts.iter().enumerate() {
                if i > 0 {
                    line.push(Seg::Text(" ".into()));
                }
                line.push(Seg::Value {
                    num: *n,
                    unit: u.clone(),
                    mode: NumMode::Result,
                });
            }
            (line, Quiet::Parts(parts.clone()))
        }
    };
    pods.push(Pod {
        title: "result".into(),
        error: false,
        body: Body::Lines(vec![result_line]),
    });

    if let Answer::Single { num, unit, .. } = &o.answer {
        // A fraction answer gets its decimal alongside.
        if let Number::Exact {
            value,
            decimal: false,
        } = num
            && !value.is_integer()
        {
            let seg = Seg::Value {
                num: *num,
                unit: unit.clone(),
                mode: NumMode::Rounded,
            };
            pods.push(Pod {
                title: "decimal".into(),
                error: false,
                body: Body::Lines(vec![Line(vec![seg])]),
            });
        }
        let alts = other_units(&o.value, unit);
        if !alts.is_empty() {
            let lines = alts
                .into_iter()
                .map(|(n, u)| {
                    Line(vec![Seg::Value {
                        num: n,
                        unit: u,
                        mode: NumMode::Rounded,
                    }])
                })
                .collect();
            pods.push(Pod {
                title: "other units".into(),
                error: false,
                body: Body::List(lines),
            });
        }
    }

    Report {
        pods,
        ok: true,
        quiet: Some(quiet),
        error_line: None,
    }
}
