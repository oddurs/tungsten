//! Normalising lexer.
//!
//! Normalisation happens *during* lexing rather than as a rewrite of the input,
//! so every token's span is a byte range into the text the user typed. Error
//! carets and the interpretation pod both depend on that.
//!
//! What gets normalised: unicode operators (`×` `÷` `−` `·`), superscript
//! exponents (`²` becomes `^ 2`), vulgar fractions (`½`), digit grouping
//! (`1,500`, `1 500` with a thin space), the `k` suffix (`1.5k`), feet and
//! inch marks after a number (`5'11"`), and trailing question marks.

use crate::error::{Error, ErrorKind};
use std::ops::Range;
use tungsten_units::Rational;

#[derive(Clone, Debug, PartialEq)]
pub enum TokKind {
    /// A number literal. `decimal` is true when written with a point or an
    /// exponent, which decides whether results show as fractions.
    Num {
        value: Rational,
        decimal: bool,
        superscript: bool,
    },
    /// A literal too large or small to hold exactly: `3.2e-53`.
    Float(f64),
    Word(String),
    Op(char),
    /// `'` directly after a number.
    Foot,
    /// `"` directly after a number.
    Inch,
    /// `'s` directly after a word: `earth's mass`.
    Possessive,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokKind,
    pub span: Range<usize>,
}

const GROUP_SPACES: [char; 2] = ['\u{2009}', '\u{202F}'];

fn vulgar(c: char) -> Option<(i128, i128)> {
    Some(match c {
        '½' => (1, 2),
        '⅓' => (1, 3),
        '⅔' => (2, 3),
        '¼' => (1, 4),
        '¾' => (3, 4),
        '⅕' => (1, 5),
        '⅖' => (2, 5),
        '⅗' => (3, 5),
        '⅘' => (4, 5),
        '⅙' => (1, 6),
        '⅚' => (5, 6),
        '⅛' => (1, 8),
        '⅜' => (3, 8),
        '⅝' => (5, 8),
        '⅞' => (7, 8),
        _ => return None,
    })
}

fn superscript(c: char) -> Option<char> {
    Some(match c {
        '⁰' => '0',
        '¹' => '1',
        '²' => '2',
        '³' => '3',
        '⁴' => '4',
        '⁵' => '5',
        '⁶' => '6',
        '⁷' => '7',
        '⁸' => '8',
        '⁹' => '9',
        '⁻' => '-',
        _ => return None,
    })
}

fn operator(c: char) -> Option<char> {
    Some(match c {
        '+' | '-' | '*' | '/' | '^' | '(' | ')' | ',' | '=' | '!' | '%' => c,
        '×' | '·' | '⋅' | '∙' | '✕' => '*',
        '÷' | '∕' => '/',
        '−' | '–' => '-',
        '[' | '{' => '(',
        ']' | '}' => ')',
        _ => return None,
    })
}

fn starts_word(c: char) -> bool {
    c.is_alphabetic() || matches!(c, '°' | '℃' | '℉' | '$' | '_')
}

fn continues_word(c: char) -> bool {
    c.is_alphabetic() || matches!(c, '°' | '_' | '₀' | '℃' | '℉')
}

struct Lexer<'a> {
    src: &'a str,
    chars: Vec<(usize, char)>,
    i: usize,
    out: Vec<Token>,
}

impl Lexer<'_> {
    fn at(&self, i: usize) -> Option<char> {
        self.chars.get(i).map(|&(_, c)| c)
    }

    fn byte(&self, i: usize) -> usize {
        self.chars.get(i).map_or(self.src.len(), |&(b, _)| b)
    }

    fn push(&mut self, kind: TokKind, from: usize, to: usize) {
        let span = self.byte(from)..self.byte(to);
        self.out.push(Token { kind, span });
    }

    fn prev_adjacent_word(&self, i: usize) -> Option<&str> {
        match self.out.last() {
            Some(Token {
                kind: TokKind::Word(w),
                span,
            }) if span.end == self.byte(i) => Some(w),
            _ => None,
        }
    }

    fn prev_is_adjacent_num(&self, i: usize) -> bool {
        matches!(
            self.out.last(),
            Some(Token { kind: TokKind::Num { .. }, span }) if span.end == self.byte(i)
        )
    }

    fn run(mut self) -> Result<Vec<Token>, Error> {
        while let Some(c) = self.at(self.i) {
            let start = self.i;
            if c.is_whitespace() {
                self.i += 1;
            } else if c.is_ascii_digit()
                || (c == '.' && self.at(self.i + 1).is_some_and(|d| d.is_ascii_digit()))
            {
                self.number()?;
            } else if let Some((n, d)) = vulgar(c) {
                self.i += 1;
                let value = Rational::new(n, d).unwrap_or(Rational::ZERO);
                self.push(
                    TokKind::Num {
                        value,
                        decimal: false,
                        superscript: false,
                    },
                    start,
                    self.i,
                );
            } else if superscript(c).is_some() {
                self.superscripts();
            } else if c == '*' && self.at(self.i + 1) == Some('*') {
                self.i += 2;
                self.push(TokKind::Op('^'), start, self.i);
            } else if let Some(op) = operator(c) {
                self.i += 1;
                self.push(TokKind::Op(op), start, self.i);
            } else if matches!(c, '\'' | '’' | '′') && self.prev_is_adjacent_num(start) {
                self.i += 1;
                self.push(TokKind::Foot, start, self.i);
            } else if matches!(c, '"' | '”' | '″') && self.prev_is_adjacent_num(start) {
                self.i += 1;
                self.push(TokKind::Inch, start, self.i);
            } else if matches!(c, '\'' | '’') && self.prev_adjacent_word(start).is_some() {
                // earth's, mars'
                let next = self.at(self.i + 1);
                let after = self.at(self.i + 2);
                if matches!(next, Some('s' | 'S')) && !after.is_some_and(char::is_alphanumeric) {
                    self.i += 2;
                    self.push(TokKind::Possessive, start, self.i);
                } else if !next.is_some_and(char::is_alphanumeric)
                    && self
                        .prev_adjacent_word(start)
                        .is_some_and(|w| w.ends_with(['s', 'S']))
                {
                    self.i += 1;
                    self.push(TokKind::Possessive, start, self.i);
                } else {
                    self.i += 1;
                }
            } else if c == '.'
                && self.prev_adjacent_word(start).is_some()
                && self.at(self.i + 1).is_some_and(char::is_alphabetic)
            {
                // earth.mass
                self.i += 1;
                self.push(TokKind::Op('.'), start, self.i);
            } else if matches!(c, '?' | '"' | '“' | '”' | '\'' | '‘' | '’' | '`') {
                // Question marks and stray quotes carry no meaning.
                self.i += 1;
            } else if c == 'Δ' || starts_word(c) {
                self.word();
            } else {
                let b = self.byte(start);
                return Err(Error::new(
                    ErrorKind::UnexpectedChar(c),
                    b..b + c.len_utf8(),
                ));
            }
        }
        Ok(self.out)
    }

    /// Integer digits, with grouping: `1,500`, `1 500 000` (plain, thin or
    /// narrow no-break spaces). A separator only groups when the digits before
    /// it form a valid group (1–3 digits first, then exactly 3), so `2 3` stays
    /// two numbers and `1234,567` is not grouped.
    fn digits(&mut self, text: &mut String) {
        let mut run = 0;
        let mut grouped = false;
        while let Some(c) = self.at(self.i) {
            if c.is_ascii_digit() {
                text.push(c);
                run += 1;
                self.i += 1;
            } else if (c == ',' || c == ' ' || GROUP_SPACES.contains(&c))
                && (if grouped {
                    run == 3
                } else {
                    (1..=3).contains(&run)
                })
                && self.group_follows(self.i + 1)
            {
                self.i += 1;
                run = 0;
                grouped = true;
            } else {
                break;
            }
        }
    }

    /// Exactly three digits and then a non-digit: a digit group.
    fn group_follows(&self, i: usize) -> bool {
        (0..3).all(|k| self.at(i + k).is_some_and(|c| c.is_ascii_digit()))
            && !self.at(i + 3).is_some_and(|c| c.is_ascii_digit())
    }

    fn number(&mut self) -> Result<(), Error> {
        let start = self.i;
        let mut text = String::new();
        let mut decimal = false;
        self.digits(&mut text);
        if self.at(self.i) == Some('.') && self.at(self.i + 1).is_some_and(|c| c.is_ascii_digit()) {
            decimal = true;
            text.push('.');
            self.i += 1;
            while let Some(c) = self.at(self.i).filter(char::is_ascii_digit) {
                text.push(c);
                self.i += 1;
            }
        }
        if matches!(self.at(self.i), Some('e' | 'E')) {
            let mut j = self.i + 1;
            if matches!(self.at(j), Some('+' | '-')) {
                j += 1;
            }
            if self.at(j).is_some_and(|c| c.is_ascii_digit())
                && !self.at(j + 1).is_some_and(char::is_alphabetic)
            {
                decimal = true;
                text.push('e');
                for k in self.i + 1..j {
                    text.push(self.at(k).unwrap_or('+'));
                }
                self.i = j;
                while let Some(c) = self.at(self.i).filter(char::is_ascii_digit) {
                    text.push(c);
                    self.i += 1;
                }
            }
        }
        let span = self.byte(start)..self.byte(self.i);
        let Some(mut value) = Rational::parse(&text) else {
            return match text.parse::<f64>() {
                Ok(x) if x.is_finite() => {
                    self.push(TokKind::Float(x), start, self.i);
                    Ok(())
                }
                _ => Err(Error::new(ErrorKind::NumberTooLarge, span)),
            };
        };
        // 1.5k
        if self.at(self.i) == Some('k') && !self.at(self.i + 1).is_some_and(char::is_alphanumeric) {
            self.i += 1;
            value = value
                .checked_mul(Rational::int(1000))
                .ok_or_else(|| Error::new(ErrorKind::NumberTooLarge, span.clone()))?;
        }
        // 2½
        if let Some((n, d)) = self.at(self.i).and_then(vulgar) {
            self.i += 1;
            let frac = Rational::new(n, d).unwrap_or(Rational::ZERO);
            value = value.checked_add(frac).unwrap_or(value);
        }
        self.push(
            TokKind::Num {
                value,
                decimal,
                superscript: false,
            },
            start,
            self.i,
        );
        Ok(())
    }

    fn superscripts(&mut self) {
        let start = self.i;
        let mut text = String::new();
        while let Some(d) = self.at(self.i).and_then(superscript) {
            text.push(d);
            self.i += 1;
        }
        self.push(TokKind::Op('^'), start, self.i);
        let (neg, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text.as_str()),
        };
        if neg {
            self.push(TokKind::Op('-'), start, self.i);
        }
        let value = digits
            .parse::<i128>()
            .map(Rational::int)
            .unwrap_or(Rational::ONE);
        self.push(
            TokKind::Num {
                value,
                decimal: false,
                superscript: true,
            },
            start,
            self.i,
        );
    }

    fn word(&mut self) {
        let start = self.i;
        let mut text = String::new();
        while let Some(c) = self.at(self.i) {
            let first = self.i == start;
            if (first && (c == 'Δ' || starts_word(c))) || continues_word(c) {
                text.push(c);
                self.i += 1;
            } else if c == '-'
                && self.at(self.i + 1).is_some_and(char::is_alphabetic)
                && text.chars().last().is_some_and(char::is_alphabetic)
            {
                // light-year, pound-force
                text.push(c);
                self.i += 1;
            } else if c.is_ascii_digit() {
                // Trailing digits belong to the word (km2, g0); digits followed
                // by more letters start a new token (2h15min).
                let mut j = self.i;
                while self.at(j).is_some_and(|d| d.is_ascii_digit()) {
                    j += 1;
                }
                if self.at(j).is_some_and(char::is_alphabetic) {
                    break;
                }
                while self.i < j {
                    text.push(self.at(self.i).unwrap_or('0'));
                    self.i += 1;
                }
                break;
            } else {
                break;
            }
        }
        self.push(TokKind::Word(text), start, self.i);
    }
}

pub fn lex(src: &str) -> Result<Vec<Token>, Error> {
    Lexer {
        src,
        chars: src.char_indices().collect(),
        i: 0,
        out: Vec::new(),
    }
    .run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn kinds(s: &str) -> Vec<TokKind> {
        lex(s).unwrap().into_iter().map(|t| t.kind).collect()
    }

    fn num(n: i128) -> TokKind {
        TokKind::Num {
            value: Rational::int(n),
            decimal: false,
            superscript: false,
        }
    }

    fn w(s: &str) -> TokKind {
        TokKind::Word(s.into())
    }

    #[test]
    fn normalises_numbers() {
        assert_eq!(kinds("1,500"), vec![num(1500)]);
        assert_eq!(kinds("1\u{202F}500"), vec![num(1500)]);
        assert_eq!(kinds("1 000 000 s")[0], num(1_000_000));
        assert_eq!(kinds("2 3"), vec![num(2), num(3)]);
        assert_eq!(
            kinds("1234,567"),
            vec![num(1234), TokKind::Op(','), num(567)]
        );
        assert_eq!(
            kinds("1.5k")[0],
            TokKind::Num {
                value: Rational::int(1500),
                decimal: true,
                superscript: false
            }
        );
        assert_eq!(kinds("h, min"), vec![w("h"), TokKind::Op(','), w("min")]);
        assert_eq!(
            kinds("2½")[0],
            TokKind::Num {
                value: Rational::new(5, 2).unwrap(),
                decimal: false,
                superscript: false
            }
        );
    }

    #[test]
    fn normalises_operators() {
        assert_eq!(
            kinds("3 × 4 ÷ 2 − 1"),
            vec![
                num(3),
                TokKind::Op('*'),
                num(4),
                TokKind::Op('/'),
                num(2),
                TokKind::Op('-'),
                num(1)
            ]
        );
        let sup = kinds("m²");
        assert_eq!(sup[0], w("m"));
        assert_eq!(sup[1], TokKind::Op('^'));
        assert!(matches!(
            sup[2],
            TokKind::Num {
                superscript: true,
                ..
            }
        ));
        assert_eq!(kinds("2**3")[1], TokKind::Op('^'));
    }

    #[test]
    fn feet_and_inches() {
        assert_eq!(
            kinds("5'11\""),
            vec![num(5), TokKind::Foot, num(11), TokKind::Inch]
        );
        assert_eq!(
            kinds("5’11”"),
            vec![num(5), TokKind::Foot, num(11), TokKind::Inch]
        );
    }

    #[test]
    fn words() {
        assert_eq!(kinds("2h15min"), vec![num(2), w("h"), num(15), w("min")]);
        assert_eq!(kinds("km2"), vec![w("km2")]);
        assert_eq!(
            kinds("°C Δ°F light-year"),
            vec![w("°C"), w("Δ°F"), w("light-year")]
        );
        assert_eq!(kinds("what is 2?"), vec![w("what"), w("is"), num(2)]);
    }

    #[test]
    fn possessives_and_dots() {
        assert_eq!(
            kinds("earth's mass"),
            vec![w("earth"), TokKind::Possessive, w("mass")]
        );
        assert_eq!(
            kinds("mars' moons"),
            vec![w("mars"), TokKind::Possessive, w("moons")]
        );
        assert_eq!(kinds("earth’s"), vec![w("earth"), TokKind::Possessive]);
        assert_eq!(
            kinds("earth.mass"),
            vec![w("earth"), TokKind::Op('.'), w("mass")]
        );
        // A foot mark is still a foot mark.
        assert_eq!(kinds("5'")[1], TokKind::Foot);
    }

    #[test]
    fn rejects_unknown_characters() {
        let e = lex("3 @ 4").unwrap_err();
        assert_eq!(e.span, 2..3);
    }

    proptest! {
        #[test]
        fn spans_are_valid_slices(s in "\\PC{0,40}") {
            if let Ok(tokens) = lex(&s) {
                for t in tokens {
                    prop_assert!(t.span.start <= t.span.end);
                    prop_assert!(s.get(t.span.clone()).is_some());
                }
            }
        }

        #[test]
        fn spans_are_valid_for_math_like_input(s in "[0-9a-z .,'\"²³½×÷−+*/^()%!°Δµ]{0,40}") {
            if let Ok(tokens) = lex(&s) {
                for t in tokens {
                    prop_assert!(s.get(t.span.clone()).is_some());
                }
            }
        }
    }
}
