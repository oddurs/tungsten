//! Reconstructs a query as it was understood, for the interpretation pod.
//!
//! Every unit comes out as its canonical symbol and every implicit operator is
//! made explicit, so the user can see exactly how their words were read.

use crate::ast::{BinOp, Expr, Node, PostOp, Query, Style};
use tungsten_units::{Number, UnitExpr};

#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    Num(Number),
    Unit(UnitExpr),
    /// An operator with its surrounding spaces: ` × `, ` + `.
    Op(&'static str),
    /// A small integer exponent, shown as a superscript where possible.
    Sup(i128),
    Word(&'static str),
    Space,
    Open,
    Close,
    Arrow,
    Comma,
}

pub fn interpret(q: &Query) -> Vec<Piece> {
    let mut out = Vec::new();
    emit(&q.expr, 0, false, &mut out);
    if !q.targets.is_empty() {
        out.push(Piece::Arrow);
        for (i, t) in q.targets.iter().enumerate() {
            if i > 0 {
                out.push(Piece::Comma);
            }
            out.push(Piece::Unit(t.unit.clone()));
        }
    }
    out
}

fn prec(n: &Node) -> u8 {
    match &n.expr {
        Expr::Bin {
            style: Style::Compound,
            ..
        } => 9,
        Expr::Bin {
            op: BinOp::Add | BinOp::Sub,
            ..
        } => 10,
        Expr::Bin { .. } => 20,
        Expr::Neg(_) => 25,
        Expr::Pow(..) => 40,
        Expr::Post(..) => 50,
        _ => 90,
    }
}

fn emit(n: &Node, parent: u8, right: bool, out: &mut Vec<Piece>) {
    let p = prec(n);
    // Compounds are always bracketed inside a larger expression.
    let compound = p == 9 && parent > 0;
    let paren = compound || p < parent || (right && p == parent && p != 90);
    if paren {
        out.push(Piece::Open);
    }
    match &n.expr {
        Expr::Num(x) => out.push(Piece::Num(*x)),
        Expr::Quantity { value, unit } => {
            if let Some(v) = value {
                out.push(Piece::Num(*v));
                // 30°, but 30 °C (SI Brochure §5.4.3).
                if unit.display(true) != "°" {
                    out.push(Piece::Space);
                }
            }
            out.push(Piece::Unit(unit.clone()));
        }
        Expr::Const(c) => out.push(Piece::Word(c.symbol())),
        Expr::Neg(x) => {
            out.push(Piece::Op("−"));
            emit(x, 25, false, out);
        }
        // Parentheses the user wrote are always kept.
        Expr::Group(x) => {
            out.push(Piece::Open);
            emit(x, 0, false, out);
            out.push(Piece::Close);
        }
        Expr::Bin {
            op,
            style,
            lhs,
            rhs,
        } => {
            let (sym, level) = match (op, style) {
                (BinOp::Add, Style::Compound) => (" + ", 10),
                (BinOp::Add, _) => (" + ", 10),
                (BinOp::Sub, _) => (" − ", 10),
                (BinOp::Mul, _) => (" × ", 20),
                (BinOp::Div, _) => (" / ", 20),
            };
            emit(lhs, level, false, out);
            out.push(Piece::Op(sym));
            emit(rhs, level, true, out);
        }
        Expr::Pow(base, exp) => {
            emit(base, 41, false, out);
            match &exp.expr {
                Expr::Num(Number::Exact { value, .. }) if value.is_integer() => {
                    out.push(Piece::Sup(value.num()));
                }
                _ => {
                    out.push(Piece::Op("^"));
                    emit(exp, 40, true, out);
                }
            }
        }
        Expr::Post(op, x) => {
            emit(x, 50, false, out);
            out.push(Piece::Op(if *op == PostOp::Percent { "%" } else { "!" }));
        }
        Expr::Call(f, args) => {
            out.push(Piece::Word(f.name()));
            out.push(Piece::Open);
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push(Piece::Comma);
                }
                emit(a, 0, false, out);
            }
            out.push(Piece::Close);
        }
    }
    if paren {
        out.push(Piece::Close);
    }
}
