//! Turns words into meanings: units, keywords, functions, constants, and the
//! entities and properties of the knowledge base.
//!
//! Multi-word phrases are matched longest-first, so `miles per hour` is one
//! unit, `melting point` one property and `divided by` one operator, before
//! any single word is looked at. A word can mean several things (`calories`
//! is a unit and a property; `mercury` a planet and an element); the resolver
//! keeps every meaning and the parser and entity chooser decide by context.

use crate::error::{Error, ErrorKind};
use crate::lex::{TokKind, Token};
use crate::suggest;
use std::ops::Range;
use tungsten_kb::{Hit, Kind, Prop};
use tungsten_units::{Number, Rational, UnitRef, lookup, max_name_words};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kw {
    In,
    To,
    As,
    Into,
    Per,
    /// `a`/`an`: an article (`a day` = 1 day) or, between operands, `per`.
    A,
    Of,
    For,
    Times,
    Plus,
    Minus,
    DividedBy,
    MultipliedBy,
    Squared,
    Cubed,
    Square,
    Cubic,
    Half,
    Twice,
    ToThePowerOf,
    SquareRootOf,
    CubeRootOf,
    And,
    How,
    Many,
    Much,
    Is,
    Are,
    There,
    Does,
    Weigh,
    Between,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Const {
    Pi,
    Tau,
    E,
}

impl Const {
    pub fn value(self) -> f64 {
        match self {
            Self::Pi => std::f64::consts::PI,
            Self::Tau => std::f64::consts::TAU,
            Self::E => std::f64::consts::E,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Pi => "π",
            Self::Tau => "τ",
            Self::E => "e",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Sqrt,
    Cbrt,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Ln,
    Log10,
    Log2,
    Exp,
    Abs,
    Round,
    Floor,
    Ceil,
}

impl Func {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sqrt => "sqrt",
            Self::Cbrt => "cbrt",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Ln => "ln",
            Self::Log10 => "log",
            Self::Log2 => "log2",
            Self::Exp => "exp",
            Self::Abs => "abs",
            Self::Round => "round",
            Self::Floor => "floor",
            Self::Ceil => "ceil",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sym {
    /// A number literal. `superscript` numbers came from `²` and never merge
    /// into fractions.
    Num {
        value: Number,
        superscript: bool,
    },
    /// `thousand`, `million`, `dozen`, `percent`.
    Magnitude(Number),
    Unit(UnitRef),
    /// `km2`: a unit with a trailing exponent.
    UnitPow(UnitRef, i128),
    Const(Const),
    Func(Func),
    Kw(Kw),
    Op(char),
    Foot,
    Inch,
    /// A word the knowledge base knows: an entity, a property, or both, and
    /// possibly also a unit.
    Name(Meaning),
    /// `'s`
    Possessive,
    /// `how tall`: a question about these properties.
    Ask(Vec<Prop>),
}

/// Everything a knowledge-base word could mean.
#[derive(Clone, Debug, PartialEq)]
pub struct Meaning {
    pub unit: Option<UnitRef>,
    pub entities: Vec<Hit>,
    pub props: Vec<Prop>,
}

impl Meaning {
    /// Entities this word can stand for as written (not shadowed).
    pub fn usable(&self) -> impl Iterator<Item = Hit> + '_ {
        self.entities.iter().copied().filter(|h| !h.shadowed)
    }

    pub fn has_entity(&self) -> bool {
        self.usable().next().is_some()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub sym: Sym,
    pub span: Range<usize>,
}

/// Matched after `the` is dropped, so `to the power of` is `to power of`.
const PHRASES: &[(&str, Kw)] = &[
    ("to power of", Kw::ToThePowerOf),
    ("raised to power of", Kw::ToThePowerOf),
    ("raised to", Kw::ToThePowerOf),
    ("square root of", Kw::SquareRootOf),
    ("cube root of", Kw::CubeRootOf),
    ("divided by", Kw::DividedBy),
    ("multiplied by", Kw::MultipliedBy),
    ("how many", Kw::How),
    ("how much", Kw::How),
];

/// `how tall is X` asks for X's height.
const ASK: &[(&str, &str)] = &[
    ("how tall", "height"),
    ("how high", "height"),
    ("how heavy", "mass"),
    ("how long", "length"),
    ("how wide", "width"),
    ("how deep", "depth"),
    ("how thick", "thickness"),
    ("how far", "distance"),
    ("how old", "age"),
    ("how fast", "speed"),
    ("how hot", "temperature"),
    ("how dense", "density"),
];

/// Leading phrases that carry no meaning.
const FILLER: &[&str] = &[
    "what is",
    "what's",
    "whats",
    "what are",
    "calculate",
    "compute",
    "convert",
    "evaluate",
    "please",
];

fn keyword(w: &str) -> Option<Kw> {
    Some(match w {
        "in" => Kw::In,
        "to" => Kw::To,
        "as" => Kw::As,
        "into" => Kw::Into,
        "per" => Kw::Per,
        "a" | "an" => Kw::A,
        "of" => Kw::Of,
        "for" => Kw::For,
        "times" | "x" => Kw::Times,
        "plus" => Kw::Plus,
        "minus" => Kw::Minus,
        "over" => Kw::DividedBy,
        "squared" => Kw::Squared,
        "cubed" => Kw::Cubed,
        "square" | "sq" => Kw::Square,
        "cubic" | "cu" => Kw::Cubic,
        "half" => Kw::Half,
        "twice" | "double" => Kw::Twice,
        "and" => Kw::And,
        "is" => Kw::Is,
        "are" => Kw::Are,
        "there" => Kw::There,
        "many" => Kw::Many,
        "much" => Kw::Much,
        "does" | "do" => Kw::Does,
        "weigh" | "weighs" => Kw::Weigh,
        "between" => Kw::Between,
        _ => return None,
    })
}

fn constant(w: &str) -> Option<Const> {
    Some(match w {
        "pi" | "π" => Const::Pi,
        "tau" | "τ" => Const::Tau,
        "e" => Const::E,
        _ => return None,
    })
}

fn function(w: &str) -> Option<Func> {
    Some(match w {
        "sqrt" => Func::Sqrt,
        "cbrt" => Func::Cbrt,
        "sin" => Func::Sin,
        "cos" => Func::Cos,
        "tan" => Func::Tan,
        "asin" | "arcsin" => Func::Asin,
        "acos" | "arccos" => Func::Acos,
        "atan" | "arctan" => Func::Atan,
        "ln" => Func::Ln,
        "log" | "log10" => Func::Log10,
        "log2" => Func::Log2,
        "exp" => Func::Exp,
        "abs" => Func::Abs,
        "round" => Func::Round,
        "floor" => Func::Floor,
        "ceil" => Func::Ceil,
        _ => return None,
    })
}

fn magnitude(w: &str) -> Option<Number> {
    let (n, decimal) = match w {
        "hundred" => (Rational::int(100), false),
        "thousand" => (Rational::int(1_000), false),
        "million" => (Rational::int(1_000_000), false),
        "billion" => (Rational::int(1_000_000_000), false),
        "trillion" => (Rational::int(1_000_000_000_000), false),
        "quadrillion" => (Rational::int(1_000_000_000_000_000), false),
        "dozen" => (Rational::int(12), false),
        "percent" | "pct" => (Rational::new(1, 100)?, true),
        "permille" => (Rational::new(1, 1000)?, true),
        _ => return None,
    };
    Some(Number::exact(n, decimal))
}

/// Every word the resolver knows that is not a unit, for suggestions.
pub fn vocabulary() -> impl Iterator<Item = &'static str> {
    [
        "in", "to", "as", "into", "per", "of", "for", "times", "plus", "minus", "over", "squared",
        "cubed", "square", "cubic", "half", "twice", "double", "pi", "tau", "sqrt", "cbrt", "sin",
        "cos", "tan", "asin", "acos", "atan", "ln", "log", "log2", "exp", "abs", "round", "floor",
        "ceil", "hundred", "thousand", "million", "billion", "trillion", "dozen", "percent",
    ]
    .into_iter()
}

/// What a word or phrase means as a unit, entity or property.
///
/// Entities whose name is shadowed by a unit (`W` is the watt before it is
/// tungsten) only count when that unit is not in play, or when `prefer` asks
/// for their kind (`--as element`).
fn meaning(words: &str, prefer: Option<Kind>) -> Option<Sym> {
    let unit = lookup(words);
    let mut entities = tungsten_kb::lookup(words);
    let props = tungsten_kb::lookup_prop(words);
    for h in &mut entities {
        // Shadowed by a keyword spelled differently (`In` is not `in`), or
        // explicitly asked for: usable.
        if unit.is_none() || prefer == Some(h.entity.kind()) {
            h.shadowed = false;
        }
    }
    let usable = entities.iter().any(|h| !h.shadowed);
    match (unit, usable || !props.is_empty()) {
        (Some(u), false) => Some(Sym::Unit(u)),
        (None, false) if entities.is_empty() => None,
        _ => Some(Sym::Name(Meaning {
            unit,
            entities,
            props,
        })),
    }
}

fn single(word: &str, prefer: Option<Kind>) -> Option<Sym> {
    let lower = word.to_lowercase();
    if let Some(k) = keyword(word) {
        return Some(Sym::Kw(k));
    }
    // --as constant e: the elementary charge, not Euler's number.
    if let Some(k) = prefer
        && tungsten_kb::lookup(word)
            .iter()
            .any(|h| h.entity.kind() == k)
        && let Some(sym) = meaning(word, prefer)
    {
        return Some(sym);
    }
    if let Some(c) = constant(word) {
        return Some(Sym::Const(c));
    }
    if let Some(f) = function(&lower) {
        return Some(Sym::Func(f));
    }
    if let Some(m) = magnitude(&lower) {
        return Some(Sym::Magnitude(m));
    }
    if let Some(sym) = meaning(word, prefer) {
        return Some(sym);
    }
    // km2, m3, s2
    let letters = word.trim_end_matches(|c: char| c.is_ascii_digit());
    if letters.len() < word.len() && !letters.is_empty() {
        if let (Some(u), Ok(e)) = (lookup(letters), word[letters.len()..].parse::<i128>()) {
            return Some(Sym::UnitPow(u, e));
        }
    }
    None
}

pub fn resolve(tokens: &[Token], prefer: Option<Kind>) -> Result<Vec<Item>, Error> {
    // `the` carries no meaning anywhere: `the mass of the earth`.
    let tokens: Vec<Token> = tokens
        .iter()
        .filter(|t| !matches!(&t.kind, TokKind::Word(w) if w.eq_ignore_ascii_case("the")))
        .cloned()
        .collect();
    // `what's`: the apostrophe is not a possessive.
    let tokens: &[Token] = match tokens.as_slice() {
        [
            Token {
                kind: TokKind::Word(w),
                ..
            },
            Token {
                kind: TokKind::Possessive,
                ..
            },
            rest @ ..,
        ] if w.eq_ignore_ascii_case("what") || w.eq_ignore_ascii_case("who") => rest,
        all => all,
    };
    let max_words = max_name_words().max(tungsten_kb::max_name_words()).max(4);
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        let word = match &t.kind {
            TokKind::Word(w) => w,
            TokKind::Num {
                value,
                decimal,
                superscript,
            } => {
                let sym = Sym::Num {
                    value: Number::exact(*value, *decimal),
                    superscript: *superscript,
                };
                out.push(Item {
                    sym,
                    span: t.span.clone(),
                });
                i += 1;
                continue;
            }
            TokKind::Float(x) => {
                let sym = Sym::Num {
                    value: Number::Approx(*x),
                    superscript: false,
                };
                out.push(Item {
                    sym,
                    span: t.span.clone(),
                });
                i += 1;
                continue;
            }
            TokKind::Op(c) => {
                out.push(Item {
                    sym: Sym::Op(*c),
                    span: t.span.clone(),
                });
                i += 1;
                continue;
            }
            TokKind::Foot => {
                out.push(Item {
                    sym: Sym::Foot,
                    span: t.span.clone(),
                });
                i += 1;
                continue;
            }
            TokKind::Inch => {
                out.push(Item {
                    sym: Sym::Inch,
                    span: t.span.clone(),
                });
                i += 1;
                continue;
            }
            TokKind::Possessive => {
                out.push(Item {
                    sym: Sym::Possessive,
                    span: t.span.clone(),
                });
                i += 1;
                continue;
            }
        };

        // Longest multi-word phrase starting here: keyword phrase or unit name.
        let mut matched = None;
        for n in (2..=max_words).rev() {
            let Some(words) = phrase(tokens, i, n) else {
                continue;
            };
            let lower = words.to_lowercase();
            if out.is_empty() && FILLER.contains(&lower.as_str()) {
                matched = Some((n, None));
                break;
            }
            if let Some((_, k)) = PHRASES.iter().find(|(p, _)| *p == lower) {
                matched = Some((n, Some(Sym::Kw(*k))));
                break;
            }
            if let Some((_, prop)) = ASK.iter().find(|(p, _)| *p == lower) {
                matched = Some((n, Some(Sym::Ask(tungsten_kb::lookup_prop(prop)))));
                break;
            }
            if let Some(sym) = meaning(&words, prefer) {
                matched = Some((n, Some(sym)));
                break;
            }
        }
        if let Some((n, sym)) = matched {
            if let Some(sym) = sym {
                let span = t.span.start..tokens[i + n - 1].span.end;
                out.push(Item { sym, span });
            }
            i += n;
            continue;
        }

        if out.is_empty() && FILLER.contains(&word.to_lowercase().as_str()) {
            i += 1;
            continue;
        }
        match single(word, prefer) {
            Some(sym) => out.push(Item {
                sym,
                span: t.span.clone(),
            }),
            None => {
                let suggestion = suggest::suggest(word);
                return Err(Error::new(
                    ErrorKind::UnknownWord {
                        word: word.clone(),
                        suggestion,
                    },
                    t.span.clone(),
                ));
            }
        }
        i += 1;
    }
    Ok(out)
}

/// `n` consecutive word tokens starting at `i`, joined by single spaces.
/// Whole numbers may appear after the first word (`boeing 747`), and a
/// possessive rejoins its word (`avogadro's number`).
fn phrase(tokens: &[Token], i: usize, n: usize) -> Option<String> {
    let slice = tokens.get(i..i + n)?;
    let mut words: Vec<String> = Vec::with_capacity(n);
    for (k, t) in slice.iter().enumerate() {
        match &t.kind {
            TokKind::Word(w) => words.push(w.clone()),
            // A phrase never ends on a possessive: `earth's` alone is not a name.
            TokKind::Possessive if k > 0 && k + 1 < n => {
                words.last_mut()?.push_str("'s");
            }
            TokKind::Num {
                value,
                decimal: false,
                superscript: false,
            } if k > 0 && value.is_integer() => words.push(value.num().to_string()),
            _ => return None,
        }
    }
    Some(words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    fn syms(s: &str) -> Vec<Sym> {
        resolve(&lex(s).unwrap(), None)
            .unwrap()
            .into_iter()
            .map(|i| i.sym)
            .collect()
    }

    #[test]
    fn phrases_win() {
        let s = syms("60 miles per hour");
        assert_eq!(s[1], Sym::Unit(lookup("mph").unwrap()));
        assert_eq!(syms("6 divided by 3")[1], Sym::Kw(Kw::DividedBy));
        assert_eq!(syms("1 fl oz")[1], Sym::Unit(lookup("floz").unwrap()));
        assert_eq!(syms("speed of light")[0], Sym::Unit(lookup("c").unwrap()));
    }

    #[test]
    fn filler_is_dropped() {
        assert_eq!(syms("what is 2").len(), 1);
        assert_eq!(syms("convert 2 km to m").len(), 4);
    }

    #[test]
    fn trailing_digit_exponents() {
        assert_eq!(syms("km2")[0], Sym::UnitPow(lookup("km").unwrap(), 2));
    }

    #[test]
    fn unknown_words_suggest() {
        let e = resolve(&lex("5 kilometers in milez").unwrap(), None).unwrap_err();
        match *e.kind {
            ErrorKind::UnknownWord { word, suggestion } => {
                assert_eq!(word, "milez");
                assert_eq!(suggestion.as_deref(), Some("miles"));
            }
            other => panic!("{other:?}"),
        }
    }
}
