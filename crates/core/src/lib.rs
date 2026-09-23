//! Lexer, resolver, parser, dimension checker and evaluator for tungsten.
//!
//! ```text
//! text → lex → resolve → parse → check → eval → convert
//! ```
//!
//! See docs/concept.md §2 and §3.

mod ast;
mod check;
mod complete;
mod entities;
mod env;
mod error;
mod eval;
mod highlight;
mod interp;
mod lex;
mod parse;
mod resolve;
mod session;
mod suggest;

pub use ast::{BinOp, Choice, Expr, Mention, Node, PostOp, Query, Style, Target};
pub use complete::{Candidate, Completion, complete};
pub use entities::Assumption;
pub use env::UserFunc;
pub use error::{Error, ErrorKind, Hint};
pub use eval::{Value, is_scale, quantity};
pub use highlight::{Class, classify};
pub use interp::{Piece, interpret};
pub use resolve::{Const, Func, Scope};
pub use session::{Binding, Session};
pub use suggest::suggest;
pub use tungsten_kb::{Entity, Kind, Prop};

use tungsten_units::{MathError, Number, Rational, UnitExpr, UnitRef};

/// What a query came to.
#[derive(Clone, Debug)]
pub enum Answer {
    /// One number in one unit. `unit` is empty for plain numbers.
    Single {
        num: Number,
        unit: UnitExpr,
        point: bool,
    },
    /// A mixed-unit answer, largest unit first: `27 h 46 min 40 s`.
    Parts(Vec<(Number, UnitExpr)>),
    /// A lone entity, to be shown as a card: `gold`.
    Card(Entity),
    /// A statement that defined a function: `f(x) = x^2`.
    Defined {
        name: String,
        params: Vec<String>,
        body: String,
    },
}

/// How to read a query.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// `--as element`: prefer this kind when a name means several things,
    /// and let names shadowed by units (`W`) mean it.
    pub prefer: Option<Kind>,
}

#[derive(Clone, Debug)]
pub struct Outcome {
    pub query: Query,
    /// The value before conversion, in SI.
    pub value: Value,
    pub answer: Answer,
    /// Names that could have meant several things, and what was assumed.
    pub assumptions: Vec<Assumption>,
    /// Every unit and knowledge-base value the query touched, for `--why`.
    pub sources: Vec<Source>,
    /// What the statement bound, in a session: `rent = …`.
    pub binding: Option<Binding>,
}

/// Something the answer depends on that came from a table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Unit(UnitRef),
    Value { entity: Entity, prop: Prop },
    Entity(Entity),
}

/// The fewest significant digits among the knowledge-base values a query
/// used, if it used any. An integer's trailing zeros do not count: NOAA's
/// "up to 330,000 pounds" has two.
pub fn precision(sources: &[Source]) -> Option<u32> {
    sources
        .iter()
        .filter_map(|s| match s {
            Source::Value { entity, prop } => entity.value(*prop),
            _ => None,
        })
        .map(|v| {
            v.digits.unwrap_or_else(|| {
                let digits = match v.num.as_rational() {
                    Some(r) if r.is_integer() => r.num().unsigned_abs().to_string(),
                    _ => return 4,
                };
                u32::try_from(digits.trim_end_matches('0').len().max(1)).unwrap_or(4)
            })
        })
        .min()
}

/// Units and values in the order they appear, each once.
fn sources(q: &Query, card: Option<Entity>) -> Vec<Source> {
    fn walk(n: &Node, out: &mut Vec<Source>) {
        let mut push = |s: Source| {
            if !out.contains(&s) {
                out.push(s);
            }
        };
        match &n.expr {
            Expr::Quantity { unit, .. } => {
                for (u, _) in unit.terms() {
                    push(Source::Unit(*u));
                }
            }
            Expr::Entity(m) | Expr::Prop(m, _) => {
                if let Some(Choice {
                    entity,
                    prop: Some(prop),
                }) = m.chosen
                {
                    push(Source::Value { entity, prop });
                }
            }
            Expr::Neg(x) | Expr::Group(x) | Expr::Post(_, x) => walk(x, out),
            Expr::Bin { lhs, rhs, .. } | Expr::Pow(lhs, rhs) => {
                walk(lhs, out);
                walk(rhs, out);
            }
            Expr::Call(_, args) | Expr::UserCall(_, args) => args.iter().for_each(|a| walk(a, out)),
            Expr::Num(_) | Expr::Const(_) | Expr::Var(_) => {}
        }
    }
    let mut out = Vec::new();
    if let Some(e) = card {
        out.push(Source::Entity(e));
        return out;
    }
    walk(&q.expr, &mut out);
    for t in &q.targets {
        for (u, _) in t.unit.terms() {
            let s = Source::Unit(*u);
            if !out.contains(&s) {
                out.push(s);
            }
        }
    }
    out
}

/// Parses, checks and evaluates one query.
pub fn evaluate(src: &str) -> Result<Outcome, Error> {
    evaluate_with(src, Options::default())
}

pub fn evaluate_with(src: &str, opts: Options) -> Result<Outcome, Error> {
    let tokens = lex::lex(src)?;
    evaluate_tokens(&tokens, src, &env::Env::default(), opts)
}

/// Evaluates tokens lexed from `src`, with a session's bindings.
fn evaluate_tokens(
    tokens: &[lex::Token],
    src: &str,
    env: &env::Env,
    opts: Options,
) -> Result<Outcome, Error> {
    let env = &env.clone();
    let items = resolve::resolve(tokens, opts.prefer, &env.scope())?;
    let mut query = parse::parse(items, src)?;
    let assumptions = entities::choose(&mut query, opts.prefer)?;
    if entities::is_card(&query)
        && let Expr::Entity(m) = &query.expr.expr
        && let Some(c) = m.chosen
    {
        return Ok(Outcome {
            value: Value::scalar(Number::ZERO),
            answer: Answer::Card(c.entity),
            sources: sources(&query, Some(c.entity)),
            query,
            assumptions,
            binding: None,
        });
    }
    check::check_query(&query, src, env)?;
    let value = eval::eval(&query.expr, env)?;
    let math = |e: MathError| Error::new(ErrorKind::Math(e), query.expr.span.clone());
    // A bare unit asks what it is: `speed of light` is 299 792 458 m/s.
    let bare = match &query.expr.expr {
        Expr::Quantity { value: None, unit }
            if query.targets.is_empty()
                && !value.dim.is_none()
                && !unit.single().is_some_and(is_scale) =>
        {
            let si = tungsten_units::coherent(&value.dim);
            (si != *unit).then_some(si)
        }
        _ => None,
    };
    let answer = match (query.targets.as_slice(), bare) {
        ([], Some(si)) => convert(&value, &si).map_err(math)?,
        ([], None) => Answer::Single {
            num: value.shown().map_err(math)?,
            unit: value.unit.clone(),
            point: value.point,
        },
        ([t], _) => convert(&value, &t.unit).map_err(math)?,
        (many, _) => parts(&value, many).map_err(math)?,
    };
    Ok(Outcome {
        sources: sources(&query, None),
        query,
        value,
        answer,
        assumptions,
        binding: None,
    })
}

/// Parses without evaluating, for fuzzing and the REPL highlighter.
pub fn parse(src: &str) -> Result<Query, Error> {
    let tokens = lex::lex(src)?;
    let items = resolve::resolve(&tokens, None, &Scope::default())?;
    parse::parse(items, src)
}

fn convert(v: &Value, target: &UnitExpr) -> Result<Answer, MathError> {
    // mpg ↔ L/100km: the target is the reciprocal dimension.
    if target.dim() != v.dim && target.dim() == v.dim.recip() {
        return Ok(Answer::Single {
            num: Number::ONE.div(v.num)?.div(target.factor())?,
            unit: target.clone(),
            point: false,
        });
    }
    if let Some(u) = target.single() {
        if v.point && is_scale(u) {
            return Ok(Answer::Single {
                num: v.in_unit(target)?,
                unit: target.clone(),
                point: true,
            });
        }
        // A difference shown on an affine scale is a Δ unit: (30 °C − 20 °C) in °F → 18 Δ°F.
        if let Some(delta) = u.delta() {
            let unit = UnitExpr::one(delta);
            return Ok(Answer::Single {
                num: v.num.div(unit.factor())?,
                unit,
                point: false,
            });
        }
    }
    Ok(Answer::Single {
        num: v.num.div(target.factor())?,
        unit: target.clone(),
        point: false,
    })
}

fn parts(v: &Value, targets: &[Target]) -> Result<Answer, MathError> {
    let negative = v.num.is_negative();
    let mut rest = v.num.abs();
    let mut out = Vec::with_capacity(targets.len());
    for (i, t) in targets.iter().enumerate() {
        let f = t.unit.factor();
        let q = rest.div(f)?;
        let n = if i + 1 == targets.len() {
            q
        } else {
            let whole = match q.as_rational() {
                Some(r) => Number::exact(Rational::int(r.floor()), false),
                // Guard against 59.99999 minutes from float error.
                None => Number::approx((q.to_f64() + 1e-9).floor())?,
            };
            rest = rest.sub(whole.mul(f)?)?;
            if rest.is_negative() {
                rest = Number::ZERO;
            }
            whole
        };
        let n = if negative && i == 0 { n.neg() } else { n };
        out.push((n, t.unit.clone()));
    }
    Ok(Answer::Parts(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single(s: &str) -> (f64, String) {
        match evaluate(s).unwrap_or_else(|e| panic!("{s}: {e:?}")).answer {
            Answer::Single { num, unit, .. } => (num.to_f64(), unit.display(false)),
            _ => panic!("not a single value"),
        }
    }

    fn exact(s: &str) -> String {
        match evaluate(s).unwrap().answer {
            Answer::Single { num, unit, .. } => format!(
                "{} {}",
                num.as_rational().expect("exact"),
                unit.display(false)
            )
            .trim()
            .to_string(),
            _ => panic!("not a single value"),
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9 * b.abs().max(1.0)
    }

    #[test]
    fn headline_examples() {
        assert_eq!(exact("5 mi in km"), "25146/3125 km");
        assert_eq!(exact("60 mph * 2h 15min"), "135 mi");
        assert_eq!(exact("1/3 + 1/6"), "1/2");
        assert_eq!(exact("1/2 km in m"), "500 m");
        assert_eq!(exact("what is 1,500 × 2²?"), "6000");
        assert_eq!(exact("20% of 80"), "16");
        assert_eq!(exact("3 ft in cm"), "2286/25 cm");
        let (v, u) = single("speed of light * 1 week in m");
        assert_eq!(v, 181_314_478_598_400.0);
        assert_eq!(u, "m");
    }

    #[test]
    fn dimensions() {
        assert_eq!(exact("sqrt(9 m^2)"), "3 m");
        let (v, u) = single("sqrt(2 m)");
        assert!(close(v, std::f64::consts::SQRT_2));
        assert_eq!(u, "m^(1/2)");
        assert_eq!(exact("5 kg * 9.81 m/s^2"), "981/20 N");
        assert_eq!(exact("3 m / 2 s"), "3/2 m/s");
        assert_eq!(exact("1 h / 30 min"), "2");
        assert_eq!(exact("3 a day"), "3 d^-1");
    }

    #[test]
    fn temperatures() {
        assert_eq!(exact("98.6 °F in °C"), "37 °C");
        assert_eq!(exact("20 °C + 5 Δ°C"), "25 °C");
        assert_eq!(exact("30 °C - 20 °C"), "10 Δ°C");
        assert_eq!(exact("300 K in °C"), "537/20 °C");
        assert_eq!(exact("-40 °C in °F"), "-40 °F");
        assert_eq!(exact("10 K in °C"), "-5263/20 °C");
        assert_eq!(exact("(30 °C - 20 °C) in °F"), "18 Δ°F");
        assert!(matches!(
            *evaluate("20 °C + 5 °C").unwrap_err().kind,
            ErrorKind::AddTemperatures { hint: Some(_) }
        ));
        assert_eq!(exact("300 K + 5 K"), "305 K");
        // Readings in products are differences.
        assert_eq!(exact("4.18 J/(g °C) * 100 g * 10 °C"), "4180 J");
        assert_eq!(exact("2 * 20 °C"), "40 Δ°C");
        assert_eq!(exact("100 J/°C * 5 Δ°C"), "500 J");
    }

    #[test]
    fn mixed_unit_answers() {
        let Answer::Parts(p) = evaluate("100000 s in h, min, s").unwrap().answer else {
            panic!()
        };
        let got: Vec<String> = p
            .iter()
            .map(|(n, u)| format!("{:?} {}", n, u.display(false)))
            .collect();
        assert_eq!(got, ["27 h", "46 min", "40d s"]);
    }

    #[test]
    fn errors() {
        let e = evaluate("3 m + 2 s").unwrap_err();
        match *e.kind {
            ErrorKind::Mismatch {
                operands,
                hint: Some(h),
                ..
            } => {
                assert_eq!(operands.len(), 2);
                assert_eq!(h.text, "3 m / 2 s");
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            *evaluate("5 km in s").unwrap_err().kind,
            ErrorKind::Convert { .. }
        ));
        assert!(matches!(
            *evaluate("1/0").unwrap_err().kind,
            ErrorKind::Math(MathError::DivideByZero)
        ));
        assert!(matches!(
            *evaluate("sin(3 m)").unwrap_err().kind,
            ErrorKind::NeedsNumber { .. }
        ));
    }

    #[test]
    fn functions() {
        let (v, _) = single("sin(30°)");
        assert!(close(v, 0.5));
        assert_eq!(exact("round(2.6 km)"), "3 km");
        assert_eq!(exact("5!"), "120");
        // Beyond i128: falls back to floating point instead of failing.
        let (v, _) = single("3.2e-53 * 2");
        assert!((v - 6.4e-53).abs() < 1e-66);
        let (v, _) = single("1e60 kg in t");
        assert!((v - 1e57).abs() / 1e57 < 1e-12);
        let (v, u) = single("sqrt(1 acre)");
        assert!(close(v, 63.614_907_234_075_23), "{v}");
        assert_eq!(u, "m");
    }
}
