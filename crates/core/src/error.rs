//! Query errors. Every error carries a span into the original input.

use crate::eval::Value;
use std::ops::Range;
use tungsten_units::{Dim, MathError};

#[derive(Clone, Debug)]
pub struct Error {
    /// Boxed: some kinds carry dimensions and hint values, and errors travel
    /// through every `Result` in the pipeline.
    pub kind: Box<ErrorKind>,
    pub span: Range<usize>,
}

impl Error {
    pub fn new(kind: ErrorKind, span: Range<usize>) -> Self {
        Self {
            kind: Box::new(kind),
            span,
        }
    }
}

/// A suggested rewrite of the query, with what it would evaluate to.
#[derive(Clone, Debug)]
pub struct Hint {
    pub text: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnexpectedChar(char),
    NumberTooLarge,
    UnknownWord {
        word: String,
        suggestion: Option<String>,
    },
    /// The parser found something it cannot use here.
    Unexpected {
        found: String,
        expected: &'static str,
    },
    UnexpectedEnd {
        expected: &'static str,
    },
    Empty,
    /// `+` or `-` between quantities of different dimensions.
    Mismatch {
        op: char,
        operands: Vec<(Range<usize>, Dim)>,
        hint: Option<Hint>,
    },
    /// Two points on a temperature scale added together.
    AddTemperatures {
        hint: Option<Hint>,
    },
    NegateTemperature,
    Convert {
        from: Dim,
        to: Dim,
    },
    /// Something that only makes sense for plain numbers got a quantity.
    NeedsNumber {
        what: &'static str,
        dim: Dim,
    },
    IrrationalPower {
        dim: Dim,
    },
    WrongArity {
        func: &'static str,
        expected: usize,
        found: usize,
    },
    FactorialDomain,
    Math(MathError),
    /// `mass of gold`: the entity has no such property.
    NoProperty {
        entity: tungsten_kb::Entity,
        prop: &'static str,
    },
    /// `3 golds`: the entity stands for no single quantity.
    NoValue {
        entity: tungsten_kb::Entity,
    },
    /// A name that only means something shadowed (asked for with `--as`).
    NotAThing,
    /// `it` before there is a previous answer.
    NoIt,
    /// `m = 5`: the name already means something that cannot be rebound.
    CannotAssign {
        name: String,
        reason: String,
    },
    /// A user function called with the wrong number of arguments.
    UserArity {
        name: String,
        expected: usize,
        found: usize,
    },
    /// A user function that calls itself too deeply.
    TooDeep {
        name: String,
    },
}
