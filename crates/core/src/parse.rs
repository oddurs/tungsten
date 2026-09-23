//! Pratt parser.
//!
//! Binding power, loosest first (docs/concept.md §2):
//!
//! ```text
//!   conversion (in/to/as)   top level only, always last
//!   + - plus minus and      10
//!   * / of per for a        20
//!   unary -                 25
//!   implicit multiplication 30   2π, 3 m / 2 s = (3 m)/(2 s)
//!   ^                       40   right-associative
//!   postfix % ! squared     50
//! ```
//!
//! Two rules sit outside the table. A fraction of two bare number literals
//! binds tightest of all, so `1/2 km` is half a kilometre. And quantities of
//! the same dimension in a row add: `2 h 15 min`, `5'11"`.

use crate::ast::{BinOp, Expr, Mention, Node, PostOp, Query, Style, Target};
use crate::error::{Error, ErrorKind};
use crate::resolve::{Func, Item, Kw, Meaning, Sym};
use std::ops::Range;
use tungsten_kb::Prop;
use tungsten_units::{Dim, Number, Rational, UnitExpr, UnitRef, lookup_symbol};

const ADD: u8 = 10;
const MUL: u8 = 20;
const NEG: u8 = 25;
const IMPLICIT: u8 = 30;
const POW: u8 = 40;
const POST: u8 = 50;

pub fn parse(items: Vec<Item>, src: &str) -> Result<Query, Error> {
    let items = merge_fractions(items);
    if items.is_empty() {
        return Err(Error::new(ErrorKind::Empty, 0..src.len()));
    }
    if let Some(q) = question(&items, src) {
        return q;
    }
    if items[0].sym == Sym::Kw(Kw::How) {
        return how_many(&items, src);
    }

    let mut errors = Vec::new();
    for i in (1..items.len()).rev() {
        if !matches!(items[i].sym, Sym::Kw(Kw::In | Kw::To | Kw::As | Kw::Into)) {
            continue;
        }
        if i + 1 == items.len() {
            continue;
        }
        match targets(&items[i + 1..], src) {
            Ok((targets, of)) => match whole(&items[..i], src) {
                // `100 in cm`: a bare number before `in` means inches.
                Ok(Node {
                    expr: Expr::Num(n),
                    span,
                }) if items[i].sym == Sym::Kw(Kw::In) && of.is_none() => {
                    let inch = UnitExpr::one(lookup_symbol("in").expect("in"));
                    let span = span.start..items[i].span.end;
                    let expr = Node::new(
                        Expr::Quantity {
                            value: Some(n),
                            unit: inch,
                        },
                        span,
                    );
                    return Ok(Query { expr, targets, of });
                }
                Ok(expr) => return Ok(Query { expr, targets, of }),
                Err(e) => errors.push(e),
            },
            Err(e) => errors.push(e),
        }
    }
    match whole(&items, src) {
        Ok(expr) => Ok(Query {
            expr,
            targets: Vec::new(),
            of: None,
        }),
        Err(e) => {
            errors.push(e);
            // The error that got furthest into the input is the useful one.
            Err(errors
                .into_iter()
                .max_by_key(|e| e.span.start)
                .expect("at least one error"))
        }
    }
}

/// The thing a question is about, as a property access.
fn ask(props: &[Prop], items: &[Item], src: &str) -> Result<Query, Error> {
    // how heavy is *a* blue whale
    let items = match items.first() {
        Some(it) if it.sym == Sym::Kw(Kw::A) => &items[1..],
        _ => items,
    };
    let node = whole(items, src)?;
    let Expr::Entity(m) = node.expr else {
        return Err(Error::new(
            ErrorKind::Unexpected {
                found: src.get(node.span.clone()).unwrap_or("").to_string(),
                expected: "a thing, like the eiffel tower",
            },
            node.span,
        ));
    };
    Ok(Query {
        expr: Node::new(Expr::Prop(m, props.to_vec()), node.span),
        targets: Vec::new(),
        of: None,
    })
}

/// Questions people type that are property accesses in disguise:
///
/// - `how tall is the eiffel tower`, `how far is the moon`
/// - `how much does a blue whale weigh`
/// - `distance from earth to moon`, `distance between earth and moon`
fn question(items: &[Item], src: &str) -> Option<Result<Query, Error>> {
    let is = |it: &Item, k: Kw| it.sym == Sym::Kw(k);
    match &items[0].sym {
        Sym::Ask(props) => {
            let mut rest = &items[1..];
            while rest
                .first()
                .is_some_and(|it| is(it, Kw::Is) || is(it, Kw::Are) || is(it, Kw::Does))
            {
                rest = &rest[1..];
            }
            // `how long does a marathon go` reads fine without its verb.
            if rest
                .last()
                .is_some_and(|it| is(it, Kw::Weigh) || is(it, Kw::Is))
            {
                rest = &rest[..rest.len() - 1];
            }
            Some(ask(props, rest, src))
        }
        // how much does X weigh
        Sym::Kw(Kw::How)
            if items.len() > 3
                && is(&items[1], Kw::Does)
                && is(&items[items.len() - 1], Kw::Weigh) =>
        {
            let mass = tungsten_kb::lookup_prop("mass");
            Some(ask(&mass, &items[2..items.len() - 1], src))
        }
        Sym::Name(m) if m.props.iter().any(|p| p.name().starts_with("distance")) => distance(items),
        _ => None,
    }
}

/// `distance from earth to moon`: the moon's "distance from earth".
/// `distance between earth and moon`: whichever of the two has the other's.
fn distance(items: &[Item]) -> Option<Result<Query, Error>> {
    let mention = |it: &Item| match &it.sym {
        Sym::Name(m) if !m.entities.is_empty() => Some(Mention {
            hits: m.entities.clone(),
            chosen: None,
            name_span: it.span.clone(),
        }),
        _ => None,
    };
    let span = items[0].span.start..items.last()?.span.end;
    let prop_query = |m: Mention, props: Vec<Prop>| {
        Some(Ok(Query {
            expr: Node::new(Expr::Prop(m, props), span.clone()),
            targets: Vec::new(),
            of: None,
        }))
    };
    let Sym::Name(first) = &items[0].sym else {
        return None;
    };
    match items {
        // distance from X (one phrase) to Y
        [_, to, y] if to.sym == Sym::Kw(Kw::To) => prop_query(mention(y)?, first.props.clone()),
        // distance between X and Y
        [_, between, x, and, y]
            if between.sym == Sym::Kw(Kw::Between) && and.sym == Sym::Kw(Kw::And) =>
        {
            let (mx, my) = (mention(x)?, mention(y)?);
            let from = |m: &Mention| {
                let name = m.hits.first()?.entity.display().to_lowercase();
                let name = name.trim_start_matches("the ");
                let props = tungsten_kb::lookup_prop(&format!("distance from {name}"));
                (!props.is_empty()).then_some(props)
            };
            // Prefer the pair where the second has a distance from the first.
            match (from(&mx), from(&my)) {
                (Some(p), _) if my.hits.iter().any(|h| h.entity.resolve_prop(&p).is_some()) => {
                    prop_query(my, p)
                }
                (_, Some(p)) if mx.hits.iter().any(|h| h.entity.resolve_prop(&p).is_some()) => {
                    prop_query(mx, p)
                }
                (Some(p), _) => prop_query(my, p),
                _ => None,
            }
        }
        _ => None,
    }
}

/// `how many feet in a mile` → `a mile in feet`;
/// `how much caffeine in 3 coffees` → `3 coffees` of caffeine.
fn how_many(items: &[Item], src: &str) -> Result<Query, Error> {
    let rest = &items[1..];
    let split = rest
        .iter()
        .position(|it| matches!(it.sym, Sym::Kw(Kw::In | Kw::Is | Kw::Are)))
        .unwrap_or(rest.len());
    let mut j = split;
    while rest.get(j).is_some_and(|it| {
        matches!(
            it.sym,
            Sym::Kw(Kw::In | Kw::Is | Kw::Are | Kw::There | Kw::Of)
        )
    }) {
        j += 1;
    }
    let expr = whole(&rest[j..], src).map_err(|e| {
        if rest[j..].is_empty() {
            let end = items.last().map_or(0, |it| it.span.end);
            Error::new(
                ErrorKind::UnexpectedEnd {
                    expected: "a quantity",
                },
                end..end,
            )
        } else {
            e
        }
    })?;
    // `how much caffeine`: a property, not a unit.
    if let [
        Item {
            sym: Sym::Name(m), ..
        },
    ] = &rest[..split]
        && !m.props.is_empty()
    {
        return Ok(Query {
            expr,
            targets: Vec::new(),
            of: Some(m.props.clone()),
        });
    }
    let (targets, of) = if split == 0 {
        (Vec::new(), None)
    } else {
        targets(&rest[..split], src)?
    };
    Ok(Query { expr, targets, of })
}

fn whole(items: &[Item], src: &str) -> Result<Node, Error> {
    let mut p = Parser { items, pos: 0, src };
    if items.is_empty() {
        return Err(Error::new(ErrorKind::Empty, 0..src.len()));
    }
    let node = p.expr(0)?;
    if let Some(it) = p.peek() {
        return Err(p.unexpected(it, "an operator"));
    }
    Ok(node)
}

/// Units after `in`, and an optional `of <property>`: `in g of caffeine`.
fn targets(items: &[Item], src: &str) -> Result<(Vec<Target>, Option<Vec<Prop>>), Error> {
    let mut p = Parser { items, pos: 0, src };
    let mut out = Vec::new();
    loop {
        let start = p.peek().map_or(src.len(), |it| it.span.start);
        let unit = p.unit_expr()?;
        out.push(Target {
            unit,
            span: start..p.last_end(),
        });
        match p.peek().map(|it| &it.sym) {
            Some(Sym::Op(',') | Sym::Kw(Kw::And)) => p.pos += 1,
            None => return Ok((out, None)),
            Some(Sym::Kw(Kw::Of)) => {
                p.pos += 1;
                return match p.bump() {
                    Some(Item {
                        sym: Sym::Name(m), ..
                    }) if !m.props.is_empty() && p.peek().is_none() => {
                        Ok((out, Some(m.props.clone())))
                    }
                    Some(it) => Err(p.unexpected(it, "a property, like caffeine")),
                    None => Err(Error::new(
                        ErrorKind::UnexpectedEnd {
                            expected: "a property, like caffeine",
                        },
                        p.end_span(),
                    )),
                };
            }
            Some(_) => {
                let it = p.peek().expect("peeked");
                return Err(p.unexpected(it, "a unit"));
            }
        }
    }
}

/// `1/2` → one number; `3 1/2` → a mixed number. Never after `^` or before
/// `^ ! %`, so `2^1/2` is 1 and `2/3^2` is 2/9.
fn merge_fractions(items: Vec<Item>) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::with_capacity(items.len());
    for (idx, it) in items.iter().enumerate() {
        let next_binds = matches!(
            items.get(idx + 1).map(|n| &n.sym),
            Some(Sym::Op('^' | '!' | '%'))
        );
        if let Sym::Num {
            value: b,
            superscript: false,
        } = it.sym
            && !next_binds
            && out.len() >= 2
            && out[out.len() - 1].sym == Sym::Op('/')
            && let Sym::Num {
                value: a,
                superscript: false,
            } = out[out.len() - 2].sym
            && !matches!(
                out.len().checked_sub(3).map(|k| &out[k].sym),
                Some(Sym::Op('^'))
            )
            && !b.is_zero()
            && let Ok(q) = a.div(b)
        {
            out.pop();
            let a_item = out.pop().expect("numerator");
            let mut span = a_item.span.start..it.span.end;
            let mut value = q;
            // 3 1/2
            if let (
                Some(Sym::Num {
                    value: whole,
                    superscript: false,
                }),
                true,
            ) = (out.last().map(|w| &w.sym), a.is_integer() && b.is_integer())
                && whole.is_integer()
                && !whole.is_negative()
                && q.partial_cmp(Number::ONE) == Some(std::cmp::Ordering::Less)
                && let Ok(mixed) = whole.add(q)
            {
                let w = out.pop().expect("whole part");
                span = w.span.start..span.end;
                value = mixed;
            }
            out.push(Item {
                sym: Sym::Num {
                    value,
                    superscript: false,
                },
                span,
            });
            continue;
        }
        out.push(it.clone());
    }
    out
}

struct Parser<'a> {
    items: &'a [Item],
    pos: usize,
    src: &'a str,
}

enum Infix {
    Bin(BinOp, u8),
    Pow,
    Post(PostOp),
    PowConst(i128),
    None,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Item> {
        self.items.get(self.pos)
    }

    fn peek_at(&self, k: usize) -> Option<&'a Item> {
        self.items.get(self.pos + k)
    }

    fn bump(&mut self) -> Option<&'a Item> {
        let it = self.items.get(self.pos);
        self.pos += 1;
        it
    }

    fn last_end(&self) -> usize {
        self.pos
            .checked_sub(1)
            .and_then(|i| self.items.get(i))
            .map_or(self.src.len(), |it| it.span.end)
    }

    fn end_span(&self) -> Range<usize> {
        let e = self.items.last().map_or(self.src.len(), |it| it.span.end);
        e..e
    }

    fn text(&self, span: &Range<usize>) -> String {
        self.src.get(span.clone()).unwrap_or("").to_string()
    }

    fn unexpected(&self, it: &Item, expected: &'static str) -> Error {
        Error::new(
            ErrorKind::Unexpected {
                found: self.text(&it.span),
                expected,
            },
            it.span.clone(),
        )
    }

    fn expect_close(&mut self) -> Result<usize, Error> {
        match self.peek() {
            Some(it) if it.sym == Sym::Op(')') => {
                self.pos += 1;
                Ok(it.span.end)
            }
            Some(it) => Err(self.unexpected(it, "`)`")),
            None => Err(Error::new(
                ErrorKind::UnexpectedEnd { expected: "`)`" },
                self.end_span(),
            )),
        }
    }

    fn infix(&self, sym: &Sym) -> Infix {
        match sym {
            Sym::Op('+') | Sym::Kw(Kw::Plus | Kw::And) => Infix::Bin(BinOp::Add, ADD),
            Sym::Op('-') | Sym::Kw(Kw::Minus) => Infix::Bin(BinOp::Sub, ADD),
            Sym::Op('*') | Sym::Kw(Kw::Times | Kw::MultipliedBy | Kw::Of | Kw::For) => {
                Infix::Bin(BinOp::Mul, MUL)
            }
            Sym::Op('/') | Sym::Kw(Kw::Per | Kw::DividedBy) => Infix::Bin(BinOp::Div, MUL),
            // `3 a day`: an article between operands means per.
            Sym::Kw(Kw::A) if self.peek_at(1).is_some_and(|n| starts_operand(&n.sym)) => {
                Infix::Bin(BinOp::Div, MUL)
            }
            Sym::Op('^') | Sym::Kw(Kw::ToThePowerOf) => Infix::Pow,
            Sym::Op('%') => Infix::Post(PostOp::Percent),
            Sym::Op('!') => Infix::Post(PostOp::Factorial),
            Sym::Kw(Kw::Squared) => Infix::PowConst(2),
            Sym::Kw(Kw::Cubed) => Infix::PowConst(3),
            _ => Infix::None,
        }
    }

    fn expr(&mut self, min_bp: u8) -> Result<Node, Error> {
        let mut lhs = self.prefix()?;
        while let Some(it) = self.peek() {
            let start = lhs.span.start;
            match self.infix(&it.sym) {
                Infix::Bin(op, lbp) => {
                    if lbp < min_bp {
                        break;
                    }
                    self.pos += 1;
                    let rhs = self.expr(lbp + 1)?;
                    let span = start..rhs.span.end;
                    let expr = Expr::Bin {
                        op,
                        style: Style::Explicit,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                    lhs = Node::new(expr, span);
                }
                Infix::Pow => {
                    if POW < min_bp {
                        break;
                    }
                    self.pos += 1;
                    let rhs = self.expr(POW)?;
                    let span = start..rhs.span.end;
                    lhs = Node::new(Expr::Pow(Box::new(lhs), Box::new(rhs)), span);
                }
                Infix::Post(op) => {
                    if POST < min_bp {
                        break;
                    }
                    self.pos += 1;
                    lhs = Node::new(Expr::Post(op, Box::new(lhs)), start..it.span.end);
                }
                Infix::PowConst(n) => {
                    if POST < min_bp {
                        break;
                    }
                    self.pos += 1;
                    let exp = Node::new(Expr::Num(Number::int(n)), it.span.clone());
                    lhs = Node::new(Expr::Pow(Box::new(lhs), Box::new(exp)), start..it.span.end);
                }
                Infix::None => {
                    if !starts_operand(&it.sym) || IMPLICIT < min_bp {
                        break;
                    }
                    if let Some(d) = compound_dim(&lhs)
                        && self.quantity_follows_with(&d)
                    {
                        let rhs = self.prefix()?;
                        let span = start..rhs.span.end;
                        let expr = Expr::Bin {
                            op: BinOp::Add,
                            style: Style::Compound,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        };
                        lhs = Node::new(expr, span);
                        continue;
                    }
                    let rhs = self.expr(IMPLICIT + 1)?;
                    let span = start..rhs.span.end;
                    let expr = Expr::Bin {
                        op: BinOp::Mul,
                        style: Style::Implicit,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                    lhs = Node::new(expr, span);
                }
            }
        }
        Ok(lhs)
    }

    /// Is the next thing a number followed by a unit of dimension `d`?
    fn quantity_follows_with(&self, d: &Dim) -> bool {
        let Some(Sym::Num { .. }) = self.peek().map(|i| &i.sym) else {
            return false;
        };
        let unit = match self.peek_at(1).map(|i| &i.sym) {
            Some(Sym::Unit(u) | Sym::UnitPow(u, 1)) => *u,
            Some(Sym::Foot) => lookup_symbol("ft").expect("ft"),
            Some(Sym::Inch | Sym::Kw(Kw::In)) => lookup_symbol("in").expect("in"),
            _ => return false,
        };
        unit.dim() == *d && !unit.is_affine()
    }

    fn prefix(&mut self) -> Result<Node, Error> {
        let Some(it) = self.peek() else {
            return Err(Error::new(
                ErrorKind::UnexpectedEnd {
                    expected: "a number or unit",
                },
                self.end_span(),
            ));
        };
        match &it.sym {
            Sym::Name(m) if !starts_unit(&it.sym) || self.of_follows() => self.name(m, it),
            Sym::Num { value, .. } => {
                self.pos += 1;
                self.quantity_from(*value, it.span.clone(), true)
            }
            Sym::Magnitude(m) => {
                self.pos += 1;
                self.quantity_from(*m, it.span.clone(), false)
            }
            Sym::Unit(_)
            | Sym::UnitPow(..)
            | Sym::Name(_)
            | Sym::Kw(Kw::Square | Kw::Cubic | Kw::In) => {
                let start = it.span.start;
                let unit = self.unit_run(true)?;
                let span = start..self.last_end();
                Ok(Node::new(Expr::Quantity { value: None, unit }, span))
            }
            Sym::Kw(Kw::A) => {
                self.pos += 1;
                let node = self.quantity_from(Number::ONE, it.span.clone(), false)?;
                Ok(node)
            }
            Sym::Kw(Kw::Half) => {
                self.pos += 1;
                let half = Number::exact(Rational::new(1, 2).expect("1/2"), false);
                Ok(Node::new(Expr::Num(half), it.span.clone()))
            }
            Sym::Kw(Kw::Twice) => {
                self.pos += 1;
                Ok(Node::new(Expr::Num(Number::int(2)), it.span.clone()))
            }
            Sym::Const(c) => {
                self.pos += 1;
                Ok(Node::new(Expr::Const(*c), it.span.clone()))
            }
            Sym::Func(f) => {
                self.pos += 1;
                self.call(*f, it.span.start)
            }
            Sym::Kw(k @ (Kw::SquareRootOf | Kw::CubeRootOf)) => {
                self.pos += 1;
                let f = if *k == Kw::SquareRootOf {
                    Func::Sqrt
                } else {
                    Func::Cbrt
                };
                let arg = self.expr(IMPLICIT)?;
                let span = it.span.start..arg.span.end;
                Ok(Node::new(Expr::Call(f, vec![arg]), span))
            }
            Sym::Op('(') => {
                self.pos += 1;
                let inner = self.expr(0)?;
                let end = self.expect_close()?;
                Ok(Node::new(Expr::Group(Box::new(inner)), it.span.start..end))
            }
            Sym::Op('-') => {
                self.pos += 1;
                // -40 °C is a negative temperature, not the negation of 40 °C.
                if let Some(Sym::Num { value, .. }) = self.peek().map(|n| &n.sym)
                    && self.peek_at(1).is_some_and(|n| starts_unit(&n.sym))
                {
                    let num = self.bump().expect("peeked");
                    return self.quantity_from(value.neg(), it.span.start..num.span.end, true);
                }
                let inner = self.expr(NEG)?;
                let span = it.span.start..inner.span.end;
                Ok(Node::new(Expr::Neg(Box::new(inner)), span))
            }
            Sym::Op('+') => {
                self.pos += 1;
                self.expr(NEG)
            }
            _ => Err(self.unexpected(it, "a number or unit")),
        }
    }

    /// Is the word after this one `of`, as in `mass of earth`?
    fn of_follows(&self) -> bool {
        self.peek_at(1).is_some_and(|n| n.sym == Sym::Kw(Kw::Of))
    }

    /// A knowledge-base word in operand position: `mass of earth`,
    /// `earth's mass`, `earth.mass`, or a bare entity (`gold`, `3 coffees`).
    fn name(&mut self, m: &'a Meaning, it: &'a Item) -> Result<Node, Error> {
        const THING: &str = "a thing, like earth or gold";
        if !m.props.is_empty() && self.of_follows() {
            self.pos += 2;
            // `mass of a banana`
            if self.peek().is_some_and(|n| n.sym == Sym::Kw(Kw::A)) {
                self.pos += 1;
            }
            let Some(ent) = self.bump() else {
                return Err(Error::new(
                    ErrorKind::UnexpectedEnd { expected: THING },
                    self.end_span(),
                ));
            };
            let Sym::Name(em) = &ent.sym else {
                return Err(self.unexpected(ent, THING));
            };
            if em.entities.is_empty() {
                return Err(self.unexpected(ent, THING));
            }
            let mention = Mention {
                hits: em.entities.clone(),
                chosen: None,
                name_span: ent.span.clone(),
            };
            let span = it.span.start..ent.span.end;
            return Ok(Node::new(Expr::Prop(mention, m.props.clone()), span));
        }
        if m.entities.is_empty() {
            // A property alone: `mass`. Of what?
            return Err(Error::new(
                ErrorKind::Unexpected {
                    found: self.text(&it.span),
                    expected: "`of` and a thing: mass of earth",
                },
                it.span.clone(),
            ));
        }
        self.pos += 1;
        let mention = Mention {
            hits: m.entities.clone(),
            chosen: None,
            name_span: it.span.clone(),
        };
        // earth radius, big mac calories
        if let Some(
            p @ Item {
                sym: Sym::Name(pm), ..
            },
        ) = self.peek()
            && !pm.props.is_empty()
            && !pm.has_entity()
        {
            self.pos += 1;
            let span = it.span.start..p.span.end;
            return Ok(Node::new(Expr::Prop(mention, pm.props.clone()), span));
        }
        if let Some(Sym::Possessive | Sym::Op('.')) = self.peek().map(|n| &n.sym) {
            return match self.peek_at(1) {
                Some(
                    p @ Item {
                        sym: Sym::Name(pm), ..
                    },
                ) if !pm.props.is_empty() => {
                    self.pos += 2;
                    let span = it.span.start..p.span.end;
                    Ok(Node::new(Expr::Prop(mention, pm.props.clone()), span))
                }
                Some(p) => Err(self.unexpected(p, "a property, like mass")),
                None => Err(Error::new(
                    ErrorKind::UnexpectedEnd {
                        expected: "a property, like mass",
                    },
                    self.end_span(),
                )),
            };
        }
        Ok(Node::new(Expr::Entity(mention), it.span.clone()))
    }

    fn call(&mut self, f: Func, start: usize) -> Result<Node, Error> {
        if self.peek().is_some_and(|n| n.sym == Sym::Op('(')) {
            self.pos += 1;
            let mut args = vec![self.expr(0)?];
            while self.peek().is_some_and(|n| n.sym == Sym::Op(',')) {
                self.pos += 1;
                args.push(self.expr(0)?);
            }
            let end = self.expect_close()?;
            return Ok(Node::new(Expr::Call(f, args), start..end));
        }
        let arg = self.expr(IMPLICIT + 1)?;
        let span = start..arg.span.end;
        Ok(Node::new(Expr::Call(f, vec![arg]), span))
    }

    /// A number, any magnitude words after it, then any units after those.
    fn quantity_from(
        &mut self,
        mut value: Number,
        span: Range<usize>,
        explicit: bool,
    ) -> Result<Node, Error> {
        let mut end = span.end;
        let mut absorbed = explicit;
        while let Some(Sym::Magnitude(m)) = self.peek().map(|n| &n.sym) {
            value = value
                .mul(*m)
                .map_err(|e| Error::new(ErrorKind::Math(e), span.clone()))?;
            end = self.bump().expect("peeked").span.end;
            absorbed = true;
        }
        let unit_next = self
            .peek()
            .is_some_and(|n| starts_unit(&n.sym) || (explicit && n.sym == Sym::Kw(Kw::In)));
        let unit = if unit_next {
            self.unit_run(true)?
        } else {
            UnitExpr::default()
        };
        if unit.is_empty() {
            if !absorbed {
                // A bare article: `a` alone is not a number.
                return match self.peek() {
                    Some(it) => Err(self.unexpected(it, "a unit")),
                    None => Err(Error::new(
                        ErrorKind::UnexpectedEnd { expected: "a unit" },
                        self.end_span(),
                    )),
                };
            }
            return Ok(Node::new(Expr::Num(value), span.start..end));
        }
        let span = span.start..self.last_end();
        Ok(Node::new(
            Expr::Quantity {
                value: Some(value),
                unit,
            },
            span,
        ))
    }

    /// Units written side by side: `N m`, `sq ft`, `m^2`, `km2`.
    fn unit_run(&mut self, allow_in: bool) -> Result<UnitExpr, Error> {
        let mut unit = UnitExpr::default();
        let mut first = true;
        while let Some(it) = self.peek() {
            let is_unit = match &it.sym {
                Sym::Kw(Kw::In) => first && allow_in,
                s => starts_unit(s),
            };
            if !is_unit {
                break;
            }
            unit = unit.mul(&self.unit_factor()?);
            first = false;
        }
        Ok(unit)
    }

    fn unit_factor(&mut self) -> Result<UnitExpr, Error> {
        let mut power = Rational::ONE;
        if let Some(Sym::Kw(k @ (Kw::Square | Kw::Cubic))) = self.peek().map(|n| &n.sym) {
            power = Rational::int(if *k == Kw::Square { 2 } else { 3 });
            self.pos += 1;
        }
        let Some(it) = self.bump() else {
            return Err(Error::new(
                ErrorKind::UnexpectedEnd { expected: "a unit" },
                self.end_span(),
            ));
        };
        let (u, e): (UnitRef, Rational) = match &it.sym {
            Sym::Unit(u) | Sym::Name(Meaning { unit: Some(u), .. }) => (*u, Rational::ONE),
            Sym::UnitPow(u, e) => (*u, Rational::int(*e)),
            Sym::Foot => (lookup_symbol("ft").expect("ft"), Rational::ONE),
            Sym::Inch | Sym::Kw(Kw::In) => (lookup_symbol("in").expect("in"), Rational::ONE),
            _ => return Err(self.unexpected(it, "a unit")),
        };
        let mut exp = e.checked_mul(power).unwrap_or(e);
        match self.peek().map(|n| &n.sym) {
            Some(Sym::Op('^')) => {
                if let Some(x) = self.unit_exponent() {
                    exp = exp.checked_mul(x).unwrap_or(exp);
                }
            }
            Some(Sym::Kw(Kw::Squared)) => {
                self.pos += 1;
                exp = exp.checked_mul(Rational::int(2)).unwrap_or(exp);
            }
            Some(Sym::Kw(Kw::Cubed)) => {
                self.pos += 1;
                exp = exp.checked_mul(Rational::int(3)).unwrap_or(exp);
            }
            _ => {}
        }
        UnitExpr::one(u)
            .pow(exp)
            .ok_or_else(|| Error::new(ErrorKind::NumberTooLarge, it.span.clone()))
    }

    /// `^2`, `^-1`, `^(1/2)` after a unit. Leaves the input alone and returns
    /// `None` if what follows `^` is not a plain rational.
    fn unit_exponent(&mut self) -> Option<Rational> {
        let save = self.pos;
        self.pos += 1;
        let neg = self.peek().is_some_and(|n| n.sym == Sym::Op('-'));
        if neg {
            self.pos += 1;
        }
        let paren = self.peek().is_some_and(|n| n.sym == Sym::Op('('));
        if paren {
            self.pos += 1;
        }
        let r = match self.bump().map(|n| &n.sym) {
            Some(Sym::Num { value, .. }) => value.as_rational(),
            _ => None,
        };
        if paren && self.bump().map(|n| &n.sym) != Some(&Sym::Op(')')) {
            self.pos = save;
            return None;
        }
        match r {
            Some(r) => Some(if neg { r.checked_neg()? } else { r }),
            None => {
                self.pos = save;
                None
            }
        }
    }

    /// Unit expressions after `in`: `km/h`, `m/s^2`, `W/(m^2 K)`, `sq ft`.
    fn unit_expr(&mut self) -> Result<UnitExpr, Error> {
        let mut acc = self.unit_term()?;
        loop {
            match self.peek().map(|n| &n.sym) {
                Some(Sym::Op('*')) => {
                    self.pos += 1;
                    acc = acc.mul(&self.unit_term()?);
                }
                Some(Sym::Op('/') | Sym::Kw(Kw::Per)) => {
                    self.pos += 1;
                    acc = acc.div(&self.unit_term()?);
                }
                Some(s) if starts_unit(s) || *s == Sym::Op('(') => {
                    acc = acc.mul(&self.unit_term()?);
                }
                _ => return Ok(acc),
            }
        }
    }

    fn unit_term(&mut self) -> Result<UnitExpr, Error> {
        match self.peek() {
            Some(it) if it.sym == Sym::Op('(') => {
                self.pos += 1;
                let inner = self.unit_expr()?;
                self.expect_close()?;
                if self.peek().is_some_and(|n| n.sym == Sym::Op('^'))
                    && let Some(x) = self.unit_exponent()
                {
                    return inner
                        .pow(x)
                        .ok_or_else(|| Error::new(ErrorKind::NumberTooLarge, it.span.clone()));
                }
                Ok(inner)
            }
            Some(it) if matches!(it.sym, Sym::Kw(Kw::In)) || starts_unit(&it.sym) => {
                self.unit_factor()
            }
            Some(it) => Err(self.unexpected(it, "a unit")),
            None => Err(Error::new(
                ErrorKind::UnexpectedEnd { expected: "a unit" },
                self.end_span(),
            )),
        }
    }
}

fn starts_unit(sym: &Sym) -> bool {
    match sym {
        // `calories` is a unit and a property; as a unit only where no entity
        // or `of` claims it.
        Sym::Name(m) => m.unit.is_some() && !m.has_entity(),
        _ => matches!(
            sym,
            Sym::Unit(_)
                | Sym::UnitPow(..)
                | Sym::Foot
                | Sym::Inch
                | Sym::Kw(Kw::Square | Kw::Cubic)
        ),
    }
}

fn starts_operand(sym: &Sym) -> bool {
    matches!(
        sym,
        Sym::Num { .. }
            | Sym::Magnitude(_)
            | Sym::Unit(_)
            | Sym::UnitPow(..)
            | Sym::Const(_)
            | Sym::Func(_)
            | Sym::Name(_)
            | Sym::Op('(')
            | Sym::Kw(
                Kw::Square | Kw::Cubic | Kw::Half | Kw::Twice | Kw::SquareRootOf | Kw::CubeRootOf
            )
    )
}

/// The dimension a compound quantity continues in: `2 h` and `2 h 15 min`
/// both continue in time.
fn compound_dim(node: &Node) -> Option<Dim> {
    match &node.expr {
        Expr::Quantity {
            value: Some(_),
            unit,
        } if unit.single().is_some() => {
            let u = unit.single()?;
            (!u.is_affine() && !unit.dim().is_none()).then(|| unit.dim())
        }
        Expr::Bin {
            style: Style::Compound,
            rhs,
            ..
        } => compound_dim(rhs),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lex::lex, resolve::resolve};

    fn q(s: &str) -> Query {
        parse(resolve(&lex(s).unwrap(), None).unwrap(), s).unwrap()
    }

    /// Compact S-expression for asserting structure.
    fn sx(n: &Node) -> String {
        match &n.expr {
            Expr::Num(x) => format!("{x:?}"),
            Expr::Quantity { value, unit } => match value {
                Some(v) => format!("[{v:?} {}]", unit.display(false)),
                None => format!("[{}]", unit.display(false)),
            },
            Expr::Const(c) => c.symbol().into(),
            Expr::Neg(x) => format!("(neg {})", sx(x)),
            Expr::Bin {
                op,
                style,
                lhs,
                rhs,
            } => {
                let o = match (op, style) {
                    (BinOp::Add, Style::Compound) => "&",
                    (BinOp::Add, _) => "+",
                    (BinOp::Sub, _) => "-",
                    (BinOp::Mul, Style::Implicit) => "·",
                    (BinOp::Mul, _) => "*",
                    (BinOp::Div, _) => "/",
                };
                format!("({o} {} {})", sx(lhs), sx(rhs))
            }
            Expr::Pow(a, b) => format!("(^ {} {})", sx(a), sx(b)),
            Expr::Post(PostOp::Percent, x) => format!("(% {})", sx(x)),
            Expr::Post(PostOp::Factorial, x) => format!("(! {})", sx(x)),
            Expr::Call(f, args) => {
                format!(
                    "({} {})",
                    f.name(),
                    args.iter().map(sx).collect::<Vec<_>>().join(" ")
                )
            }
            Expr::Group(x) => sx(x),
            Expr::Entity(m) => format!("<{}>", m.hits[0].entity.display()),
            Expr::Prop(m, p) => format!("<{} {}>", p[0].name(), m.hits[0].entity.display()),
        }
    }

    fn e(s: &str) -> String {
        sx(&q(s).expr)
    }

    #[test]
    fn precedence() {
        assert_eq!(e("1 + 2 * 3"), "(+ 1 (* 2 3))");
        assert_eq!(e("-2^2"), "(neg (^ 2 2))");
        assert_eq!(e("2^3^2"), "(^ 2 (^ 3 2))");
        assert_eq!(e("2^-1"), "(^ 2 (neg 1))");
        assert_eq!(e("2π"), "(· 2 π)");
    }

    #[test]
    fn fractions_bind_tightest() {
        assert_eq!(e("1/2 km"), "[1/2 km]");
        assert_eq!(e("3 m / 2 s"), "(/ [3 m] [2 s])");
        assert_eq!(e("2^1/2"), "(/ (^ 2 1) 2)");
        assert_eq!(e("2/3^2"), "(/ 2 (^ 3 2))");
        assert_eq!(e("3 1/2 in"), "[7/2 in]");
        assert_eq!(e("1/3 + 1/6"), "(+ 1/3 1/6)");
    }

    #[test]
    fn english_operators() {
        assert_eq!(e("3 a day"), "(/ 3 [d])");
        assert_eq!(e("20% of 80"), "(* (% 20) 80)");
        assert_eq!(e("60 miles per hour"), "[60 mph]");
        assert_eq!(e("5 km per hour"), "(/ [5 km] [h])");
        assert_eq!(e("half of 3 km"), "(* 1/2 [3 km])");
        assert_eq!(e("6 divided by 3"), "(/ 6 3)");
        // `the` is dropped before phrases match; this once broke.
        assert_eq!(e("2 to the power of 3"), "(^ 2 3)");
        assert_eq!(e("5 m squared"), "[5 m^2]");
        assert_eq!(e("2.4 million km"), "[2400000d km]");
        assert_eq!(e("5 percent"), "1/20d");
    }

    #[test]
    fn compounds() {
        assert_eq!(e("2h 15min"), "(& [2 h] [15 min])");
        assert_eq!(e("1 h 30 min 10 s"), "(& (& [1 h] [30 min]) [10 s])");
        assert_eq!(e("5'11\""), "(& [5 ft] [11 in])");
        assert_eq!(e("5 ft 11 in"), "(& [5 ft] [11 in])");
        assert_eq!(e("60 mph * 2h 15min"), "(* [60 mph] (& [2 h] [15 min]))");
    }

    #[test]
    fn units() {
        assert_eq!(e("9.81 m/s^2"), "(/ [981/100d m] [s^2])");
        assert_eq!(e("5 sq ft"), "[5 ft^2]");
        assert_eq!(e("3 km2"), "[3 km^2]");
        assert_eq!(e("5 N m"), "[5 N*m]");
        assert_eq!(e("-40 °C"), "[-40 °C]");
    }

    #[test]
    fn conversions() {
        let c = q("5 in in cm");
        assert_eq!(sx(&c.expr), "[5 in]");
        assert_eq!(c.targets[0].unit.display(false), "cm");
        let c = q("100000 s in h, min, s");
        assert_eq!(c.targets.len(), 3);
        let c = q("60 mph to km/h");
        assert_eq!(c.targets[0].unit.display(false), "km/h");
        let c = q("how many feet in a mile");
        assert_eq!(sx(&c.expr), "[1 mi]");
        assert_eq!(c.targets[0].unit.display(false), "ft");
        let c = q("5 in");
        assert!(c.targets.is_empty());
        let c = q("100 in cm");
        assert_eq!(sx(&c.expr), "[100 in]");
        assert_eq!(c.targets[0].unit.display(false), "cm");
    }

    #[test]
    fn functions() {
        assert_eq!(e("sqrt(9 m^2)"), "(sqrt [9 m^2])");
        assert_eq!(e("square root of 16"), "(sqrt 16)");
        assert_eq!(e("sin 30°"), "(sin [30 °])");
    }

    #[test]
    fn errors_point_at_the_problem() {
        let s = "5 km in 3";
        let err = parse(resolve(&lex(s).unwrap(), None).unwrap(), s).unwrap_err();
        assert_eq!(&s[err.span.clone()], "3");
        let s = "(1 + 2";
        let err = parse(resolve(&lex(s).unwrap(), None).unwrap(), s).unwrap_err();
        assert!(matches!(*err.kind, ErrorKind::UnexpectedEnd { .. }));
    }
}
