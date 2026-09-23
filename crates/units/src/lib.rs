//! Rationals, dimensions and the unit table for tungsten.
//!
//! The lowest crate in the workspace: nothing here knows about queries or
//! rendering. See docs/concept.md §3 and §4.

mod dim;
mod expr;
mod number;
mod quantity;
mod rational;
mod table;

pub use dim::{BaseDim, Dim, exponent};
pub use expr::{Parsed, UnitExpr, parse_def};
pub use number::{MathError, Number};
pub use quantity::{Quantity, coherent, describe, quantities, quantity_for};
pub use rational::Rational;
pub use table::{
    PrefixDef, System, UnitDef, UnitRef, all_forms, lookup, lookup_name, lookup_symbol,
    max_name_words, unit_count,
};
