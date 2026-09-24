//! A session: queries that remember. `rent = 2400 usd/month`, `it * 12`,
//! `f(x) = x^2 - 3x`, `f(5)`.
//!
//! Assignment and definitions are recognised from tokens before parsing, so
//! a name need not mean anything yet to be bound. A name that already means
//! something fixed (a unit, a function, a constant, a structural keyword, a
//! thing in the knowledge base) cannot be bound: `m = 5` is refused, because
//! `m` is metres. Keywords that are only English sugar (`a`, `x`) can be:
//! once `a = 2`, `a * 3` is 6.

use crate::env::{Env, UserFunc};
use crate::error::{Error, ErrorKind};
use crate::eval::{self, Value};
use crate::lex::{self, TokKind, Token};
use crate::{Answer, Options, Outcome, check, entities, parse, resolve};
use std::collections::BTreeMap;
use tungsten_units::Number;

/// What a statement bound, if anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Var(String),
    Func(String),
}

/// Keywords that structure a query; binding one would break the language.
const STRUCTURAL: &[&str] = &[
    "in", "to", "as", "into", "of", "per", "is", "are", "and", "how", "it", "does", "do",
    "between", "for", "plus", "minus", "times", "over", "squared", "cubed", "square", "cubic",
    "the",
];

#[derive(Clone, Debug, Default)]
pub struct Session {
    env: Env,
    pub options: Options,
}

impl Session {
    pub fn new(options: Options) -> Self {
        Self {
            env: Env::default(),
            options,
        }
    }

    /// Runs one statement. `;`-separated statements are split by the caller.
    pub fn run(&mut self, src: &str) -> Result<Outcome, Error> {
        let tokens = lex::lex(src)?;
        let words = |i: usize| match tokens.get(i).map(|t| &t.kind) {
            Some(TokKind::Word(w)) => Some(w.as_str()),
            _ => None,
        };
        let op = |i: usize, c: char| tokens.get(i).is_some_and(|t| t.kind == TokKind::Op(c));

        // name = expr
        if let Some(name) = words(0)
            && op(1, '=')
        {
            self.bindable(name, &tokens[0])?;
            if tokens.len() == 2 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEnd {
                        expected: "a value to remember",
                    },
                    src.len()..src.len(),
                ));
            }
            let mut outcome = self.evaluate(&tokens[2..], src)?;
            let value = shown(&outcome).map_err(|_| {
                Error::new(
                    ErrorKind::CannotAssign {
                        name: name.to_string(),
                        reason: "the right side has no single value to remember".into(),
                        instead: None,
                    },
                    tokens[2].span.start..src.len(),
                )
            })?;
            self.env.vars.insert(name.to_string(), value.clone());
            self.env.vars.insert("it".into(), value);
            outcome.binding = Some(Binding::Var(name.to_string()));
            return Ok(outcome);
        }

        // f(x, y) = expr
        if let Some(name) = words(0)
            && op(1, '(')
            && let Some(eq) = definition(&tokens)
        {
            let refuse = |reason: String, span: std::ops::Range<usize>| {
                Err(Error::new(
                    ErrorKind::CannotAssign {
                        name: name.to_string(),
                        reason,
                        instead: None,
                    },
                    span,
                ))
            };
            let head = tokens[0].span.start..tokens[eq - 1].span.end;
            let Some((params, eq)) = params(&tokens) else {
                return if eq == 3 {
                    refuse(format!("it needs a parameter: {name}(x) = …"), head)
                } else {
                    refuse(
                        format!("parameters are plain names: {name}(x, y) = …"),
                        head,
                    )
                };
            };
            if let Some(dup) = params
                .iter()
                .enumerate()
                .find_map(|(i, p)| params[..i].contains(p).then_some(p))
            {
                return refuse(format!("{dup} is a parameter twice"), head);
            }
            if tokens.len() == eq + 1 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEnd {
                        expected: "a body for the function",
                    },
                    src.len()..src.len(),
                ));
            }
            self.bindable(name, &tokens[0])?;
            for (p, t) in params.iter().zip(tokens.iter().skip(2).step_by(2)) {
                self.bindable_param(p, t)?;
            }
            let body_start = tokens.get(eq + 1).map_or(src.len(), |t| t.span.start);
            let body = src[body_start..].trim().to_string();
            // Check the body parses now, with the parameters (and the
            // function itself, for recursion) bound, rather than at first call.
            let mut probe = self.env.clone();
            for p in &params {
                probe.vars.insert(p.clone(), Value::scalar(Number::ONE));
            }
            probe.funcs.insert(
                name.to_string(),
                UserFunc {
                    params: params.clone(),
                    body: body.clone(),
                },
            );
            let items = resolve::resolve(&tokens[eq + 1..], self.options.prefer, &probe.scope())?;
            parse::parse(items, src)?;
            if calls_itself(name, &probe.funcs) {
                return Err(Error::new(
                    ErrorKind::CannotAssign {
                        name: name.to_string(),
                        reason: "it would call itself, which never ends".into(),
                        instead: None,
                    },
                    tokens[0].span.start..tokens[eq].span.end,
                ));
            }
            self.env.funcs.insert(
                name.to_string(),
                UserFunc {
                    params: params.clone(),
                    body: body.clone(),
                },
            );
            return Ok(Outcome {
                query: crate::Query {
                    expr: crate::Node::new(crate::Expr::Num(Number::ZERO), 0..src.len()),
                    targets: Vec::new(),
                    of: None,
                },
                value: Value::scalar(Number::ZERO),
                answer: Answer::Defined {
                    name: name.to_string(),
                    params,
                    body,
                },
                assumptions: Vec::new(),
                sources: Vec::new(),
                binding: Some(Binding::Func(name.to_string())),
            });
        }

        let outcome = self.evaluate(&tokens, src)?;
        if let Ok(v) = shown(&outcome) {
            self.env.vars.insert("it".into(), v);
        }
        Ok(outcome)
    }

    fn evaluate(&self, tokens: &[Token], src: &str) -> Result<Outcome, Error> {
        crate::evaluate_tokens(tokens, src, &self.env, self.options)
    }

    /// Why a name cannot be bound, if it cannot.
    fn bindable(&self, name: &str, at: &Token) -> Result<(), Error> {
        let Some((reason, suggest)) = taken(name) else {
            return Ok(());
        };
        // `t` is tonnes, but `t1` is free.
        let instead = suggest
            .then(|| {
                [format!("{name}1"), format!("{name}_")]
                    .into_iter()
                    .find(|n| taken(n).is_none())
            })
            .flatten();
        Err(Error::new(
            ErrorKind::CannotAssign {
                name: name.to_string(),
                reason,
                instead,
            },
            at.span.clone(),
        ))
    }

    fn bindable_param(&self, name: &str, at: &Token) -> Result<(), Error> {
        // Parameters may shadow variables and things, but not units.
        if let Some(u) = tungsten_units::lookup(name) {
            return Err(Error::new(
                ErrorKind::CannotAssign {
                    name: name.to_string(),
                    reason: format!("it is the unit {} ({})", u.symbol(), u.name(false)),
                    instead: None,
                },
                at.span.clone(),
            ));
        }
        Ok(())
    }

    /// Variables, `it` included, in name order.
    pub fn vars(&self) -> &BTreeMap<String, Value> {
        &self.env.vars
    }

    pub fn funcs(&self) -> &BTreeMap<String, UserFunc> {
        &self.env.funcs
    }

    /// Forgets every variable, function and `it`.
    pub fn clear(&mut self) {
        self.env = Env::default();
    }

    /// Names bound in this session, for highlighting and completion.
    pub fn scope(&self) -> crate::Scope {
        self.env.scope()
    }
}

/// Why `name` cannot be bound, and whether a similar name is worth offering
/// (not for keywords: `in1` helps no one).
fn taken(name: &str) -> Option<(String, bool)> {
    if STRUCTURAL.contains(&name.to_lowercase().as_str()) {
        return Some(("it is part of how queries are read".into(), false));
    }
    if let Some(u) = tungsten_units::lookup(name) {
        let what = format!("it is the unit {} ({})", u.symbol(), u.name(false));
        return Some((what, true));
    }
    if crate::resolve::is_builtin(name) {
        return Some(("it is a built-in function or constant".into(), true));
    }
    tungsten_kb::lookup(name)
        .into_iter()
        .find(|h| !h.shadowed)
        .map(|h| (format!("it is {}", h.entity.display()), true))
}

/// Whether calling `name` could reach `name` again. There are no
/// conditionals, so any cycle is endless.
fn calls_itself(name: &str, funcs: &BTreeMap<String, UserFunc>) -> bool {
    let callees = |f: &str| -> Vec<String> {
        let Some(uf) = funcs.get(f) else {
            return Vec::new();
        };
        let Ok(tokens) = lex::lex(&uf.body) else {
            return Vec::new();
        };
        tokens
            .into_iter()
            .filter_map(|t| match t.kind {
                TokKind::Word(w) if funcs.contains_key(&w) && !uf.params.contains(&w) => Some(w),
                _ => None,
            })
            .collect()
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut todo = callees(name);
    while let Some(f) = todo.pop() {
        if f == name {
            return true;
        }
        if seen.insert(f.clone()) {
            todo.extend(callees(&f));
        }
    }
    false
}

/// `f(…) =`, however malformed inside: the index of `=`.
fn definition(tokens: &[Token]) -> Option<usize> {
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().skip(1) {
        match t.kind {
            TokKind::Op('(') => depth += 1,
            TokKind::Op(')') => {
                depth -= 1;
                if depth == 0 {
                    return (tokens.get(i + 1)?.kind == TokKind::Op('=')).then_some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// `f(x, y) =`: the parameter names and the index of `=`.
fn params(tokens: &[Token]) -> Option<(Vec<String>, usize)> {
    let mut out = Vec::new();
    let mut i = 2;
    loop {
        match &tokens.get(i)?.kind {
            TokKind::Word(w) => out.push(w.clone()),
            _ => return None,
        }
        match &tokens.get(i + 1)?.kind {
            TokKind::Op(',') => i += 2,
            TokKind::Op(')') => {
                return (tokens.get(i + 2)?.kind == TokKind::Op('=')).then_some((out, i + 2));
            }
            _ => return None,
        }
    }
}

/// The value an outcome showed, as a value to remember: `5 km in mi` binds
/// 3.107 mi, not 5000 m.
fn shown(o: &Outcome) -> Result<Value, Error> {
    let math = |e| Error::new(ErrorKind::Math(e), o.query.expr.span.clone());
    match &o.answer {
        Answer::Single { num, unit, .. } if unit.is_empty() => Ok(Value::scalar(*num)),
        Answer::Single { num, unit, .. } => eval::quantity(*num, unit).map_err(math),
        // A thing stands for its headline value: `sun` for its mass.
        Answer::Card(e) => {
            let v = e
                .default()
                .and_then(|p| e.value(p))
                .ok_or_else(|| Error::new(ErrorKind::NotAThing, o.query.expr.span.clone()))?;
            eval::quantity(v.num, &v.unit).map_err(math)
        }
        Answer::Defined { .. } => Err(Error::new(ErrorKind::NotAThing, o.query.expr.span.clone())),
        Answer::Parts(_) => Ok(o.value.clone()),
    }
}

/// A user function's body, evaluated with its parameters bound.
pub(crate) fn eval_body(body: &str, env: &Env) -> Result<Value, Error> {
    let tokens = lex::lex(body)?;
    let items = resolve::resolve(&tokens, None, &env.scope())?;
    let mut query = parse::parse(items, body)?;
    entities::choose(&mut query, None)?;
    check::check_query(&query, body, env)?;
    eval::eval(&query.expr, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: &mut Session, q: &str) -> String {
        match s.run(q) {
            Ok(o) => match o.answer {
                Answer::Single { num, unit, .. } => {
                    format!("{} {}", num.to_f64(), unit.display(false))
                        .trim()
                        .to_string()
                }
                Answer::Defined { name, .. } => format!("defined {name}"),
                other => format!("{other:?}"),
            },
            Err(e) => format!("error {:?}", e.kind),
        }
    }

    #[test]
    fn variables_and_it() {
        let mut s = Session::default();
        assert_eq!(run(&mut s, "rent = 2400 USD/month"), "2400 USD/mo");
        assert_eq!(run(&mut s, "rent * 12 month"), "28800 USD");
        assert_eq!(run(&mut s, "it / 52 wk"), "553.8461538461538 USD/wk");
    }

    #[test]
    fn units_cannot_be_rebound() {
        let mut s = Session::default();
        assert!(
            run(&mut s, "m = 5").contains("CannotAssign"),
            "{}",
            run(&mut s, "m = 5")
        );
        assert!(run(&mut s, "in = 5").contains("CannotAssign"));
        assert!(run(&mut s, "earth = 5").contains("CannotAssign"));
        assert!(run(&mut s, "t = 5").contains("instead: Some(\"t1\")"));
        assert!(run(&mut s, "in = 5").contains("instead: None"));
        assert!(run(&mut s, "x =").contains("a value to remember"));
    }

    #[test]
    fn a_is_a_variable_once_bound() {
        let mut s = Session::default();
        assert_eq!(run(&mut s, "a = 2"), "2");
        assert_eq!(run(&mut s, "a * 3"), "6");
    }

    #[test]
    fn functions() {
        let mut s = Session::default();
        assert_eq!(run(&mut s, "f(x) = x^2 - 3x"), "defined f");
        assert_eq!(run(&mut s, "f(5)"), "10");
        assert_eq!(run(&mut s, "area(r) = pi r^2"), "defined area");
        assert!(run(&mut s, "area(2 m)").starts_with("12.56"));
        assert_eq!(run(&mut s, "add(len, wid) = len * wid + 1"), "defined add");
        assert_eq!(run(&mut s, "add(3, 4)"), "13");
        assert!(run(&mut s, "add(3)").contains("UserArity"));
        assert!(run(&mut s, "g(x) = x").contains("CannotAssign"));
        assert!(run(&mut s, "k(2) = 3").contains("plain names"));
        assert!(run(&mut s, "k() = 3").contains("needs a parameter"));
        assert!(run(&mut s, "k(x, x) = x").contains("twice"));
        assert!(run(&mut s, "k(x) =").contains("a body"));
    }

    #[test]
    fn recursion_is_refused() {
        let mut s = Session::default();
        assert!(run(&mut s, "loop(x) = loop(x) + 1").contains("CannotAssign"));
        run(&mut s, "f(x) = x");
        run(&mut s, "k(x) = f(x) + 1");
        assert!(run(&mut s, "f(x) = k(x)").contains("CannotAssign"));
        assert_eq!(run(&mut s, "k(1)"), "2");
    }

    #[test]
    fn deep_chains_are_bounded() {
        let mut s = Session::default();
        run(&mut s, "f0(x) = x + 1");
        for i in 1..=20 {
            run(&mut s, &format!("f{i}(x) = f{}(x)", i - 1));
        }
        assert_eq!(run(&mut s, "f10(1)"), "2");
        assert!(run(&mut s, "f20(1)").contains("TooDeep"));
    }

    #[test]
    fn it_before_anything() {
        let mut s = Session::default();
        assert!(run(&mut s, "it * 2").contains("NoIt"));
    }

    #[test]
    fn conversions_bind_what_was_shown() {
        let mut s = Session::default();
        run(&mut s, "dist = 5 km in mi");
        assert_eq!(s.vars()["dist"].unit.display(false), "mi");
    }
}
