//! The parsed form of a query.

use crate::resolve::{Const, Func};
use std::ops::Range;
use tungsten_kb::{Entity, Hit, Prop};
use tungsten_units::{Number, UnitExpr};

#[derive(Clone, Debug)]
pub struct Query {
    pub expr: Node,
    /// `in km`, `to h, min, s`. Empty when no conversion was asked for.
    pub targets: Vec<Target>,
    /// `in g of caffeine`: which property bare entities stand for.
    pub of: Option<Vec<Prop>>,
}

#[derive(Clone, Debug)]
pub struct Target {
    pub unit: UnitExpr,
    pub span: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub expr: Expr,
    pub span: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// How an operator was written, which the interpretation pod reproduces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Explicit,
    /// Juxtaposition: `2π`.
    Implicit,
    /// Same-dimension quantities in a row: `2 h 15 min`, `5'11"`.
    Compound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostOp {
    Percent,
    Factorial,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Num(Number),
    /// A number with units, or a bare unit (`km` alone is one kilometre).
    Quantity {
        value: Option<Number>,
        unit: UnitExpr,
    },
    Const(Const),
    Neg(Box<Node>),
    Bin {
        op: BinOp,
        style: Style,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Pow(Box<Node>, Box<Node>),
    Post(PostOp, Box<Node>),
    Call(Func, Vec<Node>),
    /// Parentheses the user wrote.
    Group(Box<Node>),
    /// A bare mention: `coffee` in `3 coffees`, `gold` alone. Stands for the
    /// entity's default property (or the query's `of` property).
    Entity(Mention),
    /// `mass of earth`, `earth's mass`, `earth.mass`.
    Prop(Mention, Vec<Prop>),
}

/// A word that names one or more entities, and which one was chosen.
#[derive(Clone, Debug)]
pub struct Mention {
    pub hits: Vec<Hit>,
    /// Filled in by the entity chooser, before checking.
    pub chosen: Option<Choice>,
    /// The span of the entity's name alone.
    pub name_span: Range<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Choice {
    pub entity: Entity,
    /// The property whose value this mention stands for. `None` for a lone
    /// entity shown as a card.
    pub prop: Option<Prop>,
}

impl Node {
    pub fn new(expr: Expr, span: Range<usize>) -> Self {
        Self { expr, span }
    }
}
