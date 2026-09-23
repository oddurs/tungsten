//! Number formatting (docs/concept.md §6).
//!
//! - Exact integers are never rounded: `712 800`.
//! - Fractions stay fractions when the input was fractions: `1/2`.
//! - Exact terminating decimals print in full up to 12 digits: `8.04672`.
//! - Everything else rounds to 4 significant figures (or `--sig N`), keeping
//!   every integer digit below a million (`217 261`) and switching to
//!   scientific notation outside 0.001–999 999: `1.813×10¹⁴`.
//! - Integer parts of five or more digits are grouped with a narrow
//!   no-break space (SI style), or commas under `--plain`.

use tungsten_pods::NumMode;
use tungsten_units::{Number, Rational};

#[derive(Clone, Copy, Debug)]
pub struct Fmt {
    pub sig: Option<u32>,
    pub fancy: bool,
}

const DEFAULT_SIG: u32 = 4;
const MAX_EXACT_DIGITS: usize = 12;
/// Exact integers print in full up to this many digits as a result...
const MAX_INT_DIGITS: usize = 12;
/// ...and up to this many as an alternative (below a million).
const MAX_INT_DIGITS_ROUNDED: usize = 6;
const MAX_FRACTION_DEN: i128 = 1_000_000;

pub fn number(n: Number, mode: NumMode, f: Fmt) -> String {
    let s = unsigned(n, mode, f);
    match s.strip_prefix('-') {
        Some(rest) if f.fancy => format!("−{rest}"),
        _ => s,
    }
}

fn unsigned(n: Number, mode: NumMode, f: Fmt) -> String {
    let sig = f.sig.unwrap_or(DEFAULT_SIG);
    let Number::Exact { value: r, decimal } = n else {
        return rounded(
            n.to_f64(),
            if mode == NumMode::Literal { 10 } else { sig },
            f.fancy,
        );
    };
    let limit = if mode == NumMode::Rounded {
        MAX_INT_DIGITS_ROUNDED
    } else {
        MAX_INT_DIGITS
    };
    if r.is_integer()
        && (mode == NumMode::Literal || r.num().unsigned_abs().to_string().len() <= limit)
    {
        return group(&r.num().to_string(), f.fancy);
    }
    // A large exact integer keeps its significant digits: 5.9722×10²⁴, not
    // 5.972×10²⁴ (and not 5 972 200 000 000 000 000 000 000).
    if r.is_integer() && mode == NumMode::Result && f.sig.is_none() {
        let digits = r.num().unsigned_abs().to_string();
        let sig_digits = digits.trim_end_matches('0').len().max(1);
        if sig_digits <= MAX_EXACT_DIGITS {
            return rounded(
                r.to_f64(),
                sig_digits.max(DEFAULT_SIG as usize) as u32,
                f.fancy,
            );
        }
    }
    match mode {
        NumMode::Literal => {
            if !decimal {
                return fraction(r);
            }
            exact_decimal(r)
                .map_or_else(|| rounded(r.to_f64(), 10, f.fancy), |s| group(&s, f.fancy))
        }
        NumMode::Result => {
            if !decimal && r.den() <= MAX_FRACTION_DEN {
                return fraction(r);
            }
            if f.sig.is_none()
                && let Some(s) = exact_decimal(r)
                && significant(&s) <= MAX_EXACT_DIGITS
            {
                // Tiny exact values keep every digit, in scientific form:
                // 6.6743×10⁻¹¹, not 0.000000000066743.
                if r.abs() < Rational::new(1, 1000).unwrap_or(Rational::ZERO) {
                    let n = significant(&s).max(1) as u32;
                    return rounded(r.to_f64(), n, f.fancy);
                }
                return group(&s, f.fancy);
            }
            rounded(r.to_f64(), sig, f.fancy)
        }
        NumMode::Rounded => rounded(r.to_f64(), sig, f.fancy),
    }
}

/// The bare value for `-q`: no grouping, full precision.
pub fn quiet(n: Number, sig: Option<u32>) -> String {
    if let Some(s) = sig {
        return rounded(n.to_f64(), s, false).replace(',', "");
    }
    match n {
        Number::Exact { value, .. } if value.is_integer() => value.num().to_string(),
        Number::Exact { value, .. } => exact_decimal(value)
            .filter(|s| s.len() <= 32)
            .unwrap_or_else(|| format!("{}", value.to_f64())),
        Number::Approx(x) => format!("{x}"),
    }
}

fn fraction(r: Rational) -> String {
    format!("{}/{}", r.num(), r.den())
}

/// The full decimal expansion, if it terminates and fits.
pub fn exact_decimal(r: Rational) -> Option<String> {
    if !r.is_terminating() {
        return None;
    }
    let mut den = r.den();
    let (mut twos, mut fives) = (0u32, 0u32);
    while den % 2 == 0 {
        den /= 2;
        twos += 1;
    }
    while den % 5 == 0 {
        den /= 5;
        fives += 1;
    }
    let k = twos.max(fives);
    let scale = 10i128.checked_pow(k)?;
    let scaled = r.num().checked_mul(scale)?.checked_div(r.den())?;
    let neg = scaled < 0;
    let digits = scaled.unsigned_abs().to_string();
    let k = k as usize;
    let body = if k == 0 {
        digits
    } else if digits.len() > k {
        format!(
            "{}.{}",
            &digits[..digits.len() - k],
            &digits[digits.len() - k..]
        )
    } else {
        format!("0.{}{}", "0".repeat(k - digits.len()), digits)
    };
    Some(if neg { format!("-{body}") } else { body })
}

fn significant(s: &str) -> usize {
    s.chars()
        .filter(char::is_ascii_digit)
        .skip_while(|&c| c == '0')
        .count()
}

/// Rounds to `sig` significant figures.
pub fn rounded(x: f64, sig: u32, fancy: bool) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let sig = sig.clamp(1, 17) as usize;
    let sci = format!("{:.*e}", sig - 1, x);
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    if !(-3..6).contains(&exp) {
        let m = trim_zeros(mantissa);
        return if fancy {
            format!("{m}×10{}", superscript(exp))
        } else {
            format!("{m}e{exp}")
        };
    }
    let decimals = (sig as i32 - 1 - exp).max(0) as usize;
    group(&trim_zeros(&format!("{x:.decimals$}")), fancy)
}

fn trim_zeros(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.into()
    }
}

fn superscript(n: i32) -> String {
    n.to_string()
        .chars()
        .map(|c| match c {
            '-' => '⁻',
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            _ => '⁹',
        })
        .collect()
}

/// Groups an integer part of five or more digits in threes.
fn group(s: &str, fancy: bool) -> String {
    let (sign, rest) = s.strip_prefix('-').map_or(("", s), |r| ("-", r));
    let (int, frac) = rest
        .split_once('.')
        .map_or((rest, None), |(a, b)| (a, Some(b)));
    if int.len() < 5 {
        return s.to_string();
    }
    let sep = if fancy { '\u{202F}' } else { ',' };
    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(sep);
        }
        out.push(c);
    }
    match frac {
        Some(f) => format!("{sign}{out}.{f}"),
        None => format!("{sign}{out}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FANCY: Fmt = Fmt {
        sig: None,
        fancy: true,
    };
    const PLAIN: Fmt = Fmt {
        sig: None,
        fancy: false,
    };

    fn q(s: &str, decimal: bool) -> Number {
        Number::exact(Rational::parse(s).unwrap(), decimal)
    }

    fn nb(s: &str) -> String {
        s.replace('\u{202F}', " ")
    }

    #[test]
    fn results() {
        assert_eq!(
            number(q("25146/3125", true), NumMode::Result, FANCY),
            "8.04672"
        );
        assert_eq!(number(q("1/2", false), NumMode::Result, FANCY), "1/2");
        assert_eq!(number(q("9/2", true), NumMode::Result, FANCY), "4.5");
        assert_eq!(
            nb(&number(q("712800", true), NumMode::Result, FANCY)),
            "712 800"
        );
        assert_eq!(number(q("6000", false), NumMode::Result, FANCY), "6000");
        assert_eq!(number(q("340/9", true), NumMode::Result, FANCY), "37.78");
        assert_eq!(
            number(Number::Approx(1.813_144_785_984e14), NumMode::Result, FANCY),
            "1.813×10¹⁴"
        );
        assert_eq!(
            number(Number::Approx(1.813_144_785_984e14), NumMode::Result, PLAIN),
            "1.813e14"
        );
        assert_eq!(
            number(Number::Approx(3.0e-5), NumMode::Rounded, FANCY),
            "3×10⁻⁵"
        );
        assert_eq!(number(q("-40", true), NumMode::Result, FANCY), "−40");
        assert_eq!(number(q("-40", true), NumMode::Result, PLAIN), "-40");
        assert_eq!(
            number(q("181314478598400", true), NumMode::Result, FANCY),
            "1.813×10¹⁴"
        );
        assert_eq!(
            number(q("8589934592", true), NumMode::Rounded, FANCY),
            "8.59×10⁹"
        );
        let r = |s: &str| number(q(s, true), NumMode::Result, FANCY);
        assert_eq!(r("5972200000000000000000000"), "5.9722×10²⁴");
        assert_eq!(r("0.0000000000667430"), "6.6743×10⁻¹¹");
        assert_eq!(r("0.00125"), "0.00125");
    }

    #[test]
    fn rounding_keeps_integer_digits() {
        assert_eq!(nb(&rounded(217_261.44, 4, true)), "217 261");
        assert_eq!(rounded(217.261_44, 4, true), "217.3");
        assert_eq!(rounded(0.001_234_56, 4, true), "0.001235");
        assert_eq!(rounded(9.9996, 4, true), "10");
        assert_eq!(rounded(1_234_567.0, 4, false), "1.235e6");
        assert_eq!(rounded(12_345.6, 4, false), "12,346");
    }

    #[test]
    fn sig_override() {
        let f = Fmt {
            sig: Some(3),
            fancy: true,
        };
        assert_eq!(number(q("25146/3125", true), NumMode::Result, f), "8.05");
        assert_eq!(number(q("1/2", false), NumMode::Result, f), "1/2");
    }

    #[test]
    fn quiet_is_bare() {
        assert_eq!(quiet(q("25146/3125", true), None), "8.04672");
        assert_eq!(quiet(q("2286/25", true), None), "91.44");
        assert_eq!(quiet(q("712800", true), None), "712800");
        assert_eq!(quiet(q("1/3", false), None), "0.3333333333333333");
        assert_eq!(quiet(q("25146/3125", true), Some(3)), "8.05");
    }
}
