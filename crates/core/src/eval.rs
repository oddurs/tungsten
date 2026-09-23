//! Evaluation of a type-checked query.
//!
//! Values are held in SI base units. The [`UnitExpr`] on a value only says how
//! to show it, so display decisions can never change an answer.

use crate::ast::{BinOp, Choice, Expr, Node, PostOp};
use crate::env::{Env, MAX_DEPTH};
use crate::error::{Error, ErrorKind};
use crate::resolve::Func;
use tungsten_units::{Dim, MathError, Number, Rational, UnitExpr, UnitRef, coherent};

#[derive(Clone, Debug)]
pub struct Value {
    /// Magnitude in SI base units (kelvin for temperatures).
    pub num: Number,
    pub dim: Dim,
    /// Units to display in.
    pub unit: UnitExpr,
    /// A point on a temperature scale (20 °C, 300 K) rather than a
    /// difference (5 Δ°C). Only points convert with an offset.
    pub point: bool,
    /// The number exactly as written in `unit`, while `unit` is unchanged.
    /// Converting 196.96657 u through SI kilograms overflows exact
    /// arithmetic; showing it in u should not.
    pub given: Option<Number>,
}

impl Value {
    pub fn scalar(num: Number) -> Self {
        Self {
            num,
            dim: Dim::NONE,
            unit: UnitExpr::default(),
            point: false,
            given: None,
        }
    }

    /// The number to show next to `unit`.
    pub fn in_unit(&self, unit: &UnitExpr) -> Result<Number, MathError> {
        if let Some(g) = self.given
            && *unit == self.unit
        {
            return Ok(g);
        }
        match unit.single().filter(|u| self.point && is_scale(*u)) {
            Some(u) => {
                let off = u.offset().unwrap_or(Number::ZERO);
                self.num.sub(off)?.div(u.factor())
            }
            None => self.num.div(unit.factor()),
        }
    }

    /// The number to show next to this value's own unit.
    pub fn shown(&self) -> Result<Number, MathError> {
        self.in_unit(&self.unit)
    }
}

/// A number in a unit, as a value: points on temperature scales included.
pub fn quantity(v: Number, unit: &UnitExpr) -> Result<Value, MathError> {
    Ok(match unit.single().filter(|u| is_scale(*u)) {
        Some(u) => {
            let off = u.offset().unwrap_or(Number::ZERO);
            Value {
                num: v.mul(u.factor())?.add(off)?,
                dim: unit.dim(),
                unit: unit.clone(),
                point: true,
                given: Some(v),
            }
        }
        None => Value {
            num: v.mul(unit.factor())?,
            dim: unit.dim(),
            unit: unit.clone(),
            point: false,
            given: Some(v),
        },
    })
}

/// The value a chosen entity stands for, shown in its property's display
/// unit (melting points in °C, caffeine in mg).
pub fn chosen(c: Choice) -> Result<Value, MathError> {
    let prop = c.prop.ok_or(MathError::NotReal)?;
    let v = c.entity.value(prop).ok_or(MathError::NotReal)?;
    let mut value = quantity(v.num, &v.unit)?;
    if let Some(show) = prop
        .show()
        .filter(|s| s.dim() == value.dim && *s != value.unit)
    {
        value.unit = show;
        value.given = None;
    }
    Ok(value)
}

/// A temperature scale unit: K, °C, °F, °R — but not Δ°C or Δ°F.
pub fn is_scale(u: UnitRef) -> bool {
    u.dim() == Dim::base(tungsten_units::BaseDim::Temperature) && !u.def().display.starts_with('Δ')
}

/// Chooses a display unit that actually matches the dimension, and prefers
/// SI when simplification left fractional exponents behind.
fn settle(unit: UnitExpr, dim: &Dim) -> UnitExpr {
    let fractional = unit.terms().iter().any(|(_, e)| !e.is_integer());
    let dim_integral = dim.0.iter().all(|e| e.is_integer());
    if unit.dim() != *dim || (fractional && dim_integral) {
        coherent(dim)
    } else {
        unit
    }
}

/// A reading on an affine scale (20 °C) taken as a difference (20 Δ°C).
///
/// Multiplying or dividing a thermometer reading only makes sense for a
/// difference: `4.18 J/(g °C) * 100 g * 10 °C` means ten degrees of warming,
/// not 283.15 K.
fn as_difference(v: &Value) -> Result<Value, MathError> {
    match v.unit.single().filter(|u| v.point && u.is_affine()) {
        Some(u) => {
            let unit = u.delta().map_or_else(|| coherent(&v.dim), UnitExpr::one);
            let num = v.num.sub(u.offset().unwrap_or(Number::ZERO))?;
            Ok(Value {
                num,
                dim: v.dim,
                unit,
                point: false,
                given: None,
            })
        }
        None => Ok(v.clone()),
    }
}

fn math(span: &std::ops::Range<usize>) -> impl Fn(MathError) -> Error + '_ {
    move |e| Error::new(ErrorKind::Math(e), span.clone())
}

pub fn eval(node: &Node, env: &Env) -> Result<Value, Error> {
    let m = math(&node.span);
    Ok(match &node.expr {
        Expr::Num(n) => Value::scalar(*n),
        Expr::Quantity { value, unit } => {
            quantity(value.unwrap_or(Number::ONE), unit).map_err(&m)?
        }
        Expr::Var(name) => env
            .vars
            .get(name)
            .cloned()
            .ok_or_else(|| Error::new(ErrorKind::NoIt, node.span.clone()))?,
        Expr::UserCall(name, args) => {
            if let Some(v) = env.memo.borrow().get(&(node as *const Node as usize)) {
                return Ok(v.clone());
            }
            let f = env
                .funcs
                .get(name)
                .ok_or_else(|| Error::new(ErrorKind::NoIt, node.span.clone()))?;
            if args.len() != f.params.len() {
                return Err(Error::new(
                    ErrorKind::UserArity {
                        name: name.clone(),
                        expected: f.params.len(),
                        found: args.len(),
                    },
                    node.span.clone(),
                ));
            }
            if env.depth >= MAX_DEPTH {
                return Err(Error::new(
                    ErrorKind::TooDeep { name: name.clone() },
                    node.span.clone(),
                ));
            }
            let mut inner = env.clone();
            inner.depth += 1;
            for (p, a) in f.params.iter().zip(args) {
                inner.vars.insert(p.clone(), eval(a, env)?);
            }
            // An error inside the body is reported at the call.
            crate::session::eval_body(&f.body, &inner)
                .map_err(|e| Error::new(*e.kind, node.span.clone()))?
        }
        Expr::Entity(mention) | Expr::Prop(mention, _) => {
            let c = mention
                .chosen
                .ok_or_else(|| Error::new(ErrorKind::NotAThing, node.span.clone()))?;
            chosen(c).map_err(&m)?
        }
        Expr::Const(c) => Value::scalar(Number::approx(c.value()).map_err(&m)?),
        Expr::Neg(x) => {
            let v = eval(x, env)?;
            Value {
                num: v.num.neg(),
                given: v.given.map(Number::neg),
                ..v
            }
        }
        Expr::Group(x) => eval(x, env)?,
        Expr::Bin { op, lhs, rhs, .. } => {
            let a = eval(lhs, env)?;
            let b = eval(rhs, env)?;
            binary(*op, &a, &b).map_err(|e| {
                let span = if e == MathError::DivideByZero {
                    &rhs.span
                } else {
                    &node.span
                };
                Error::new(ErrorKind::Math(e), span.clone())
            })?
        }
        Expr::Pow(base, exp) => {
            let b = as_difference(&eval(base, env)?).map_err(&m)?;
            let e = eval(exp, env)?;
            let num = b.num.pow(e.num).map_err(&m)?;
            match e.num.as_rational() {
                Some(r) => {
                    let dim = b.dim.pow(r).ok_or(Error::new(
                        ErrorKind::Math(MathError::Overflow),
                        node.span.clone(),
                    ))?;
                    let unit = b.unit.pow(r).unwrap_or_default().simplify();
                    Value {
                        num,
                        unit: settle(unit, &dim),
                        dim,
                        point: false,
                        given: None,
                    }
                }
                // Only dimensionless bases reach here (the checker ensures it).
                None => Value::scalar(num),
            }
        }
        Expr::Post(PostOp::Percent, x) => {
            let v = eval(x, env)?;
            let hundred = Number::int(100);
            Value::scalar(v.num.div(hundred).map_err(&m)?.with_decimal(true))
        }
        Expr::Post(PostOp::Factorial, x) => {
            let v = eval(x, env)?;
            Value::scalar(factorial(v.num, &node.span)?)
        }
        Expr::Call(f, args) => {
            let v = eval(&args[0], env)?;
            call(*f, &v).map_err(&m)?
        }
    })
}

pub fn binary(op: BinOp, a: &Value, b: &Value) -> Result<Value, MathError> {
    Ok(match op {
        BinOp::Add | BinOp::Sub => {
            let num = if op == BinOp::Add {
                a.num.add(b.num)?
            } else {
                a.num.sub(b.num)?
            };
            let (unit, point) = match (a.point, b.point, op) {
                // 30 °C − 20 °C is a difference: 10 Δ°C.
                (true, true, BinOp::Sub) => {
                    let u = a.unit.single();
                    let delta = u.and_then(|u| u.delta()).map(UnitExpr::one);
                    (delta.unwrap_or_else(|| a.unit.clone()), false)
                }
                (true, _, _) => (a.unit.clone(), true),
                (false, true, BinOp::Add) => (b.unit.clone(), true),
                _ => {
                    let u = if a.unit.is_empty() {
                        b.unit.clone()
                    } else {
                        a.unit.clone()
                    };
                    (u, false)
                }
            };
            Value {
                num,
                dim: a.dim,
                unit: settle(unit, &a.dim),
                point,
                given: None,
            }
        }
        BinOp::Mul | BinOp::Div => {
            let (a, b) = (as_difference(a)?, as_difference(b)?);
            let (num, dim, unit) = if op == BinOp::Mul {
                (a.num.mul(b.num)?, a.dim.mul(&b.dim), a.unit.mul(&b.unit))
            } else {
                (a.num.div(b.num)?, a.dim.div(&b.dim), a.unit.div(&b.unit))
            };
            let dim = dim.ok_or(MathError::Overflow)?;
            Value {
                num,
                dim,
                unit: settle(unit.simplify(), &dim),
                point: false,
                given: None,
            }
        }
    })
}

fn factorial(n: Number, span: &std::ops::Range<usize>) -> Result<Number, Error> {
    let fail = |k: ErrorKind| Error::new(k, span.clone());
    let Some(r) = n
        .as_rational()
        .filter(|r| r.is_integer() && !r.is_negative())
    else {
        return Err(fail(ErrorKind::FactorialDomain));
    };
    // 171! overflows f64; stop well before looping on absurd inputs.
    if r.num() > 170 {
        return Err(fail(ErrorKind::Math(MathError::Overflow)));
    }
    let mut acc = Number::ONE;
    for i in 2..=r.num() {
        acc = acc
            .mul(Number::int(i))
            .map_err(|e| fail(ErrorKind::Math(e)))?;
    }
    Ok(acc)
}

fn call(f: Func, v: &Value) -> Result<Value, MathError> {
    let scalar = |x: f64| Number::approx(x).map(Value::scalar);
    let x = v.num.to_f64();
    let rooted = |n: i128| -> Result<Value, MathError> {
        let v = as_difference(v)?;
        let r = Rational::new(1, n).ok_or(MathError::Overflow)?;
        let num = v.num.pow(Number::exact(r, false))?;
        let dim = v.dim.pow(r).ok_or(MathError::Overflow)?;
        let unit = v.unit.pow(r).unwrap_or_default().simplify();
        Ok(Value {
            num,
            unit: settle(unit, &dim),
            dim,
            point: false,
            given: None,
        })
    };
    // round/floor/ceil work in the unit the value is shown in.
    let in_unit = |g: fn(Rational) -> Option<Rational>, h: fn(f64) -> f64| {
        let shown = v.shown()?;
        let r = shown.map(g, h)?;
        let num = r.mul(v.unit.factor())?;
        let num = match v.unit.single().filter(|u| v.point && is_scale(*u)) {
            Some(u) => r.mul(u.factor())?.add(u.offset().unwrap_or(Number::ZERO))?,
            None => num,
        };
        Ok(Value {
            num,
            given: Some(r),
            ..v.clone()
        })
    };
    match f {
        Func::Sqrt => rooted(2),
        Func::Cbrt => rooted(3),
        // sin(180°) is 0, not 1.2×10⁻¹⁶: float noise at exact angles is snapped.
        Func::Sin => scalar(snap(x.sin())),
        Func::Cos => scalar(snap(x.cos())),
        Func::Tan => {
            if x.cos().abs() < 1e-12 {
                return Err(MathError::NotReal);
            }
            scalar(snap(x.tan()))
        }
        Func::Asin | Func::Acos | Func::Atan => {
            let r = match f {
                Func::Asin => x.asin(),
                Func::Acos => x.acos(),
                _ => x.atan(),
            };
            let mut out = scalar(r)?;
            out.unit = UnitExpr::parse("rad").unwrap_or_default();
            Ok(out)
        }
        Func::Ln | Func::Log10 | Func::Log2 => {
            if x <= 0.0 {
                return Err(MathError::NotReal);
            }
            scalar(match f {
                Func::Ln => x.ln(),
                Func::Log10 => x.log10(),
                _ => x.log2(),
            })
        }
        Func::Exp => scalar(x.exp()),
        Func::Abs => Ok(Value {
            num: v.num.abs(),
            given: v.given.map(Number::abs),
            ..v.clone()
        }),
        Func::Round => in_unit(round_half_away, f64::round),
        Func::Floor => in_unit(|r| Some(Rational::int(r.floor())), f64::floor),
        Func::Ceil => in_unit(
            |r| Some(Rational::int(-(r.checked_neg()?.floor()))),
            f64::ceil,
        ),
    }
}

/// Within 10⁻¹² of a whole number, it is that number.
fn snap(x: f64) -> f64 {
    let r = x.round();
    if (x - r).abs() < 1e-12 { r } else { x }
}

fn round_half_away(r: Rational) -> Option<Rational> {
    let half = Rational::new(1, 2)?;
    if r.is_negative() {
        Some(Rational::int(
            -(r.checked_neg()?.checked_add(half)?.floor()),
        ))
    } else {
        Some(Rational::int(r.checked_add(half)?.floor()))
    }
}
