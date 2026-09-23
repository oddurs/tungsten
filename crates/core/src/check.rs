//! Dimension checking, before anything is evaluated.
//!
//! Every node gets a dimension. Mismatches are reported here with the spans
//! of both operands, so the error pod can draw brackets under them.

use crate::ast::{BinOp, Expr, Node, PostOp, Query};
use crate::env::Env;
use crate::error::{Error, ErrorKind, Hint};
use crate::eval::{self, is_scale};
use crate::resolve::Func;
use tungsten_units::{Dim, Number, Rational, UnitExpr, quantity_for};

#[derive(Clone, Copy, Debug)]
struct Ty {
    dim: Dim,
    /// A point on a temperature scale.
    point: bool,
    /// On an affine scale (°C, °F), where adding two points is meaningless.
    affine: bool,
}

impl Ty {
    fn plain(dim: Dim) -> Self {
        Self {
            dim,
            point: false,
            affine: false,
        }
    }
}

pub fn check_query(q: &Query, src: &str, env: &Env) -> Result<(), Error> {
    let ty = check(&q.expr, src, env)?;
    for t in &q.targets {
        // mpg ↔ L/100km: reciprocal dimensions convert by inverting.
        let reciprocal =
            q.targets.len() == 1 && t.unit.dim() == ty.dim.recip() && !ty.dim.is_none();
        if t.unit.dim() != ty.dim && !reciprocal {
            return Err(Error::new(
                ErrorKind::Convert {
                    from: ty.dim,
                    to: t.unit.dim(),
                },
                t.span.clone(),
            ));
        }
    }
    Ok(())
}

fn text<'a>(src: &'a str, n: &Node) -> &'a str {
    src.get(n.span.clone()).unwrap_or("").trim()
}

fn check(node: &Node, src: &str, env: &Env) -> Result<Ty, Error> {
    let span = node.span.clone();
    Ok(match &node.expr {
        Expr::Num(_) | Expr::Const(_) => Ty::plain(Dim::NONE),
        // Variables and calls have a value already; its dimension is theirs.
        Expr::Var(_) | Expr::UserCall(..) => {
            let v = eval::eval(node, env)?;
            if let Expr::UserCall(..) = node.expr {
                env.memo
                    .borrow_mut()
                    .insert(node as *const Node as usize, v.clone());
            }
            match v.unit.single().filter(|u| is_scale(*u) && v.point) {
                Some(u) => Ty {
                    dim: v.dim,
                    point: true,
                    affine: u.is_affine(),
                },
                None => Ty::plain(v.dim),
            }
        }
        Expr::Entity(m) | Expr::Prop(m, _) => {
            let v = m
                .chosen
                .and_then(|c| eval::chosen(c).ok())
                .ok_or_else(|| Error::new(ErrorKind::NotAThing, span.clone()))?;
            match v.unit.single().filter(|u| is_scale(*u)) {
                Some(u) => Ty {
                    dim: v.dim,
                    point: true,
                    affine: u.is_affine(),
                },
                None => Ty::plain(v.dim),
            }
        }
        Expr::Quantity { unit, .. } => match unit.single().filter(|u| is_scale(*u)) {
            Some(u) => Ty {
                dim: unit.dim(),
                point: true,
                affine: u.is_affine(),
            },
            None => Ty::plain(unit.dim()),
        },
        Expr::Group(x) => check(x, src, env)?,
        Expr::Neg(x) => {
            let t = check(x, src, env)?;
            if t.affine {
                return Err(Error::new(ErrorKind::NegateTemperature, span));
            }
            t
        }
        Expr::Bin {
            op: op @ (BinOp::Add | BinOp::Sub),
            lhs,
            rhs,
            ..
        } => {
            let a = check(lhs, src, env)?;
            let b = check(rhs, src, env)?;
            if a.dim != b.dim {
                let sym = if *op == BinOp::Add { '+' } else { '-' };
                return Err(Error::new(
                    ErrorKind::Mismatch {
                        op: sym,
                        operands: vec![(lhs.span.clone(), a.dim), (rhs.span.clone(), b.dim)],
                        hint: mismatch_hint(lhs, rhs, src, env),
                    },
                    span,
                ));
            }
            if *op == BinOp::Add && a.point && b.point && (a.affine || b.affine) {
                return Err(Error::new(
                    ErrorKind::AddTemperatures {
                        hint: temperature_hint(lhs, rhs, src, env),
                    },
                    span,
                ));
            }
            match (a.point, b.point, op) {
                (true, true, BinOp::Sub) => Ty::plain(a.dim),
                (true, _, _) => a,
                (false, true, BinOp::Add) => b,
                _ => Ty::plain(a.dim),
            }
        }
        Expr::Bin { op, lhs, rhs, .. } => {
            let a = check(lhs, src, env)?;
            let b = check(rhs, src, env)?;
            let dim = if *op == BinOp::Mul {
                a.dim.mul(&b.dim)
            } else {
                a.dim.div(&b.dim)
            };
            Ty::plain(dim.ok_or_else(|| Error::new(ErrorKind::NumberTooLarge, span))?)
        }
        Expr::Pow(base, exp) => {
            let b = check(base, src, env)?;
            let e = check(exp, src, env)?;
            if !e.dim.is_none() {
                return Err(Error::new(
                    ErrorKind::NeedsNumber {
                        what: "an exponent",
                        dim: e.dim,
                    },
                    exp.span.clone(),
                ));
            }
            if b.dim.is_none() {
                Ty::plain(Dim::NONE)
            } else {
                let r = eval::eval(exp, env)?.num.as_rational();
                let Some(r) = r else {
                    return Err(Error::new(ErrorKind::IrrationalPower { dim: b.dim }, span));
                };
                Ty::plain(
                    b.dim
                        .pow(r)
                        .ok_or_else(|| Error::new(ErrorKind::NumberTooLarge, span))?,
                )
            }
        }
        Expr::Post(op, x) => {
            let t = check(x, src, env)?;
            if !t.dim.is_none() {
                let what = if *op == PostOp::Percent {
                    "a percentage"
                } else {
                    "a factorial"
                };
                return Err(Error::new(
                    ErrorKind::NeedsNumber { what, dim: t.dim },
                    x.span.clone(),
                ));
            }
            Ty::plain(Dim::NONE)
        }
        Expr::Call(f, args) => {
            if args.len() != 1 {
                return Err(Error::new(
                    ErrorKind::WrongArity {
                        func: f.name(),
                        expected: 1,
                        found: args.len(),
                    },
                    span,
                ));
            }
            let t = check(&args[0], src, env)?;
            match f {
                Func::Sqrt | Func::Cbrt => {
                    let r = Rational::new(1, if *f == Func::Sqrt { 2 } else { 3 }).expect("root");
                    Ty::plain(
                        t.dim
                            .pow(r)
                            .ok_or_else(|| Error::new(ErrorKind::NumberTooLarge, span))?,
                    )
                }
                Func::Abs | Func::Round | Func::Floor | Func::Ceil => t,
                _ => {
                    if !t.dim.is_none() {
                        return Err(Error::new(
                            ErrorKind::NeedsNumber {
                                what: f.name(),
                                dim: t.dim,
                            },
                            args[0].span.clone(),
                        ));
                    }
                    Ty::plain(Dim::NONE)
                }
            }
        }
    })
}

/// `3 m + 2 s` → `did you mean 3 m / 2 s (= 1.5 m/s)?`, when the division or
/// product of the operands is a named quantity.
fn mismatch_hint(lhs: &Node, rhs: &Node, src: &str, env: &Env) -> Option<Hint> {
    let a = eval::eval(lhs, env).ok()?;
    let b = eval::eval(rhs, env).ok()?;
    for (op, sym) in [(BinOp::Div, "/"), (BinOp::Mul, "×")] {
        let v = eval::binary(op, &a, &b).ok()?;
        if quantity_for(&v.dim).is_some() {
            return Some(Hint {
                text: format!("{} {sym} {}", text(src, lhs), text(src, rhs)),
                value: v,
            });
        }
    }
    None
}

/// `20 °C + 5 °C` → `did you mean 20 °C + 5 Δ°C (= 25 °C)?`
fn temperature_hint(lhs: &Node, rhs: &Node, src: &str, env: &Env) -> Option<Hint> {
    let Expr::Quantity { value, unit } = &rhs.expr else {
        return None;
    };
    let u = unit.single()?;
    let delta = u.delta()?;
    let v = value.unwrap_or(Number::ONE);
    let a = eval::eval(lhs, env).ok()?;
    let d = eval::Value {
        num: v.mul(delta.factor()).ok()?,
        dim: delta.dim(),
        unit: UnitExpr::one(delta),
        point: false,
        given: None,
    };
    let sum = eval::binary(BinOp::Add, &a, &d).ok()?;
    let shown = match v.as_rational() {
        Some(r) if r.is_integer() => r.to_string(),
        _ => format!("{}", v.to_f64()),
    };
    Some(Hint {
        text: format!("{} + {shown} {}", text(src, lhs), delta.symbol()),
        value: sum,
    })
}
