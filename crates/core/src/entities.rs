//! Choosing which entity a name means, from context.
//!
//! "mercury" is a planet and an element. `mass of mercury` needs something
//! with a mass, `3 coffees` something with a default value, and `mercury`
//! alone anything at all. Among what fits, the kind order in the knowledge
//! base decides (stars, planets and moons before elements, items, constants),
//! unless `--as <kind>` asks. When more than one thing fitted, the choice is
//! recorded as an [`Assumption`] so the answer can say what it assumed.

use crate::ast::{Choice, Expr, Mention, Node, Query};
use crate::error::{Error, ErrorKind};
use std::ops::Range;
use tungsten_kb::{Entity, Kind, Prop};

/// A name that could have meant several things, and what it was taken as.
#[derive(Clone, Debug)]
pub struct Assumption {
    pub span: Range<usize>,
    pub chosen: Entity,
    /// The other things it could have meant, in preference order.
    pub others: Vec<Entity>,
}

/// What a mention has to supply.
#[derive(Clone, Copy)]
enum Need<'a> {
    /// Anything: a lone entity shown as a card.
    Card,
    /// A value: the query's `of` property, else the entity's default.
    Value(Option<&'a [Prop]>),
    /// One of these properties.
    Prop(&'a [Prop]),
}

/// The query's lone entity, when it is nothing else: `gold`, `the moon`.
pub fn is_card(q: &Query) -> bool {
    q.targets.is_empty() && q.of.is_none() && matches!(q.expr.expr, Expr::Entity(_))
}

pub fn choose(q: &mut Query, prefer: Option<Kind>) -> Result<Vec<Assumption>, Error> {
    let mut out = Vec::new();
    if is_card(q) {
        let span = q.expr.span.clone();
        if let Expr::Entity(m) = &mut q.expr.expr {
            settle(m, Need::Card, prefer, &span, &mut out)?;
        }
        return Ok(out);
    }
    let of = q.of.clone();
    walk(&mut q.expr, of.as_deref(), prefer, &mut out)?;
    Ok(out)
}

fn walk(
    n: &mut Node,
    of: Option<&[Prop]>,
    prefer: Option<Kind>,
    out: &mut Vec<Assumption>,
) -> Result<(), Error> {
    let span = n.span.clone();
    match &mut n.expr {
        Expr::Entity(m) => settle(m, Need::Value(of), prefer, &span, out),
        Expr::Prop(m, props) => {
            let props = props.clone();
            settle(m, Need::Prop(&props), prefer, &span, out)
        }
        Expr::Neg(x) | Expr::Group(x) | Expr::Post(_, x) => walk(x, of, prefer, out),
        Expr::Bin { lhs, rhs, .. } | Expr::Pow(lhs, rhs) => {
            walk(lhs, of, prefer, out)?;
            walk(rhs, of, prefer, out)
        }
        Expr::Call(_, args) | Expr::UserCall(_, args) => {
            for a in args {
                walk(a, of, prefer, out)?;
            }
            Ok(())
        }
        Expr::Num(_) | Expr::Quantity { .. } | Expr::Const(_) | Expr::Var(_) => Ok(()),
    }
}

/// The property `e` would supply for `need`, if any.
fn supplies(e: Entity, need: Need<'_>) -> Option<Option<Prop>> {
    match need {
        Need::Card => Some(None),
        Need::Value(Some(of)) => e.resolve_prop(of).map(Some),
        Need::Value(None) => e.default().map(Some),
        Need::Prop(props) => e.resolve_prop(props).map(Some),
    }
}

fn settle(
    m: &mut Mention,
    need: Need<'_>,
    prefer: Option<Kind>,
    span: &Range<usize>,
    out: &mut Vec<Assumption>,
) -> Result<(), Error> {
    let candidates: Vec<Entity> = m
        .hits
        .iter()
        .filter(|h| !h.shadowed || Some(h.entity.kind()) == prefer)
        .map(|h| h.entity)
        .collect();
    let mut fits: Vec<(Entity, Option<Prop>)> = candidates
        .iter()
        .filter_map(|&e| supplies(e, need).map(|p| (e, p)))
        .collect();
    // Preference order: the kind order, stable within a kind.
    fits.sort_by_key(|(e, _)| e.kind());
    fits.dedup_by_key(|(e, _)| *e);

    if let Some(k) = prefer
        && let Some(&(entity, prop)) = fits.iter().find(|(e, _)| e.kind() == k)
    {
        // Asked for by kind: nothing was assumed.
        m.chosen = Some(Choice { entity, prop });
        return Ok(());
    }
    let Some(&(entity, prop)) = fits.first() else {
        let Some(&first) = candidates.first() else {
            // Only shadowed meanings, and none asked for.
            return Err(Error::new(ErrorKind::NotAThing, m.name_span.clone()));
        };
        return Err(Error::new(missing(first, need), span.clone()));
    };
    m.chosen = Some(Choice { entity, prop });
    if fits.len() > 1 {
        out.push(Assumption {
            span: m.name_span.clone(),
            chosen: entity,
            others: fits[1..].iter().map(|(e, _)| *e).collect(),
        });
    }
    Ok(())
}

fn missing(e: Entity, need: Need<'_>) -> ErrorKind {
    match need {
        Need::Prop(props) | Need::Value(Some(props)) => ErrorKind::NoProperty {
            entity: e,
            prop: props[0].name(),
        },
        Need::Value(None) | Need::Card => ErrorKind::NoValue { entity: e },
    }
}
