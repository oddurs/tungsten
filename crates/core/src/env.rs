//! What a session has bound: variables, `it`, and user functions.

use crate::eval::Value;
use crate::resolve::Scope;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// `f(x) = x^2 - 3x`: parameter names and the body as written. The body is
/// parsed again at each call, with the parameters bound as variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserFunc {
    pub params: Vec<String>,
    pub body: String,
}

#[derive(Debug, Default)]
pub struct Env {
    /// Includes `it`, the previous answer, once there is one.
    pub vars: BTreeMap<String, Value>,
    pub funcs: BTreeMap<String, UserFunc>,
    /// How deep in user-function calls evaluation is.
    pub depth: usize,
    /// User-call values computed while type-checking one query, keyed by
    /// node address, so evaluation does not repeat them. Checking a call
    /// evaluates it; without this, nested calls would cost 2^depth.
    pub(crate) memo: RefCell<BTreeMap<usize, Value>>,
}

/// A clone is for a new query (or a call's body), so it starts a new memo:
/// node addresses from another query mean nothing.
impl Clone for Env {
    fn clone(&self) -> Self {
        Self {
            vars: self.vars.clone(),
            funcs: self.funcs.clone(),
            depth: self.depth,
            memo: RefCell::default(),
        }
    }
}

/// User functions may call each other this deep, and no deeper. Without
/// conditionals, recursion never ends, so only chains of calls need room.
pub const MAX_DEPTH: usize = 16;

impl Env {
    pub fn scope(&self) -> Scope {
        Scope {
            vars: self.vars.keys().cloned().collect(),
            funcs: self.funcs.keys().cloned().collect(),
        }
    }
}
