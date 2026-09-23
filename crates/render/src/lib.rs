//! Layout, number formatting and the filament-glow theme for tungsten pods.
//!
//! ```text
//!
//!   ◆ result
//!   │ 135 mi
//!
//!   ◆ other units
//!   │ 217.3 km  ·  237 600 yd  ·  217 261 m
//!
//!   ─────────────────────────────── W74 · 0.4ms
//! ```

mod num;
mod theme;

pub use num::{Fmt, exact_decimal, number, quiet, rounded};
pub use theme::{FILAMENT, Style, Theme};

use std::time::Duration;
use tungsten_pods::{Body, Line, NumMode, Pod, Quiet, Report, Seg};
use tungsten_units::{Number, Rational, UnitExpr};
use unicode_width::UnicodeWidthStr;

/// Below this width, lists go one item per line.
pub const NARROW: usize = 60;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Terminal columns.
    pub width: usize,
    /// ANSI colour.
    pub color: bool,
    /// Unicode glyphs, superscripts and thin-space grouping; `--plain` turns
    /// this off for pure ASCII.
    pub fancy: bool,
    pub sig: Option<u32>,
    /// Shown in the footer; `None` in snapshots so they stay stable.
    pub elapsed: Option<Duration>,
}

impl Options {
    fn fmt(&self) -> Fmt {
        Fmt {
            sig: self.sig,
            fancy: self.fancy,
        }
    }
}

type Spans = Vec<(String, Style)>;

fn width(spans: &Spans) -> usize {
    spans.iter().map(|(t, _)| t.width()).sum()
}

fn superscript(n: i128, fancy: bool) -> String {
    if !fancy {
        return format!("^{n}");
    }
    tungsten_units::exponent(Rational::int(n), true)
}

/// `3 per day`, `2 cups`, `135 mi`.
fn value(num: Number, unit: &UnitExpr, mode: NumMode, o: &Options, out: &mut Spans) {
    // Only answers are bold; numbers echoed from the query are not.
    let style = if mode == NumMode::Literal {
        Style::Plain
    } else {
        Style::Value
    };
    out.push((number(num, mode, o.fmt()), style));
    if unit.is_empty() {
        return;
    }
    // 30°, but 30 °C (SI Brochure §5.4.3).
    if unit.display(true) != "°" {
        out.push((" ".into(), Style::Plain));
    }
    if let [(u, e)] = unit.terms()
        && *e == Rational::int(-1)
    {
        out.push(("per ".into(), Style::Plain));
        out.push((u.name(false), Style::Unit));
        return;
    }
    if let Some(u) = unit.single()
        && u.prefix().is_none()
        && u.def().display == u.def().name
    {
        // Units with no symbol read as words: 1 cup, 2 cups.
        let one = num.as_rational() == Some(Rational::ONE);
        out.push((u.name(!one), Style::Unit));
        return;
    }
    out.push((unit.display(o.fancy), Style::Unit));
}

fn line(l: &Line, o: &Options) -> Spans {
    let mut out = Spans::new();
    for s in &l.0 {
        match s {
            Seg::Text(t) => out.push((t.clone(), Style::Plain)),
            Seg::Dim(t) => out.push((t.clone(), Style::Dim)),
            Seg::Error(t) => out.push((t.clone(), Style::Error)),
            Seg::Value { num, unit, mode } => value(*num, unit, *mode, o, &mut out),
            Seg::Unit(u) => out.push((u.display(o.fancy), Style::Unit)),
            Seg::Sup(n) => out.push((superscript(*n, o.fancy), Style::Plain)),
        }
    }
    if !o.fancy {
        for (t, _) in &mut out {
            *t = asciify(t);
        }
    }
    out
}

fn asciify(s: &str) -> String {
    s.replace('×', "*")
        .replace('≈', "~")
        .replace('−', "-")
        .replace('→', "->")
        .replace('·', "*")
        .replace('┬', "+")
        .replace('─', "-")
        .replace('│', "|")
        .replace('└', "`")
}

/// Greedy word wrap that keeps styles; continuation lines indent by two.
/// Greedy word wrap that keeps styles. Breaks only at ordinary spaces, so a
/// word split across styles (`135` + ` ` + `mi` is two words, `2²` is one)
/// and no-break spaces stay together. Continuation lines indent by two.
fn wrap(spans: Spans, avail: usize) -> Vec<Spans> {
    let spans = if width(&spans) <= avail {
        spans
    } else {
        // Words: runs of styled fragments between ordinary spaces.
        let mut words: Vec<(Spans, usize)> = vec![(Spans::new(), 0)];
        for (text, style) in spans {
            for (i, part) in text.split(' ').enumerate() {
                if i > 0 {
                    words.push((Spans::new(), 0));
                }
                if !part.is_empty() {
                    let word = words.last_mut().expect("word");
                    word.1 += part.width();
                    word.0.push((part.to_string(), style));
                }
            }
        }
        let mut lines: Vec<Spans> = vec![Spans::new()];
        let mut w = 0;
        let mut first = true;
        for (word, ww) in words {
            let line = lines.last_mut().expect("line");
            if !first && w + 1 + ww > avail && w > 2 {
                lines.push(vec![("  ".into(), Style::Plain)]);
                w = 2;
            } else if !first {
                line.push((" ".into(), Style::Plain));
                w += 1;
            }
            lines.last_mut().expect("line").extend(word);
            w += ww;
            first = false;
        }
        return lines.into_iter().map(unbreak).collect();
    };
    vec![unbreak(spans)]
}

/// No-break spaces have done their job once lines are decided.
fn unbreak(spans: Spans) -> Spans {
    spans
        .into_iter()
        .map(|(t, s)| (t.replace('\u{a0}', " "), s))
        .collect()
}

fn body(b: &Body, o: &Options, avail: usize) -> Vec<Spans> {
    match b {
        Body::Lines(ls) => ls.iter().flat_map(|l| wrap(line(l, o), avail)).collect(),
        Body::Pre(ls) => ls.iter().map(|l| line(l, o)).collect(),
        Body::List(items) => {
            let items: Vec<Spans> = items.iter().map(|l| line(l, o)).collect();
            if o.width < NARROW {
                return items;
            }
            let sep = if o.fancy { "  ·  " } else { "  -  " };
            let mut lines = vec![Spans::new()];
            for item in items {
                let cur = lines.last_mut().expect("line");
                if !cur.is_empty() {
                    if width(cur) + sep.width() + width(&item) > avail {
                        lines.push(item);
                        continue;
                    }
                    cur.push((sep.into(), Style::Dim));
                }
                lines.last_mut().expect("line").extend(item);
            }
            lines
        }
    }
}

pub fn render(r: &Report, o: &Options) -> String {
    let t = FILAMENT;
    let (rule, dot) = if o.fancy { ("─", "·") } else { ("-", "-") };
    let (mut out, content) = pods(&r.pods, o);
    out.insert(0, '\n');

    if let Some(f) = &r.footnote {
        out.push_str("  ");
        t.paint(Style::Dim, f, o.color, &mut out);
        out.push_str("\n\n");
    }

    let label = match o.elapsed {
        Some(d) => format!("W74 {dot} {}", elapsed(d)),
        None => "W74".into(),
    };
    let rule_len = content
        .clamp(24, o.width.saturating_sub(2).max(24))
        .saturating_sub(label.width() + 1);
    out.push_str("  ");
    t.paint(Style::Dim, &rule.repeat(rule_len), o.color, &mut out);
    out.push(' ');
    t.paint(Style::Dim, &label, o.color, &mut out);
    out.push_str("\n\n");
    out
}

/// Pods alone, with no footer: the REPL's `:pods on`, and meta-commands.
pub fn render_pods(r: &Report, o: &Options) -> String {
    let (mut out, _) = pods(&r.pods, o);
    if let Some(f) = &r.footnote {
        out.push_str("  ");
        FILAMENT.paint(Style::Dim, f, o.color, &mut out);
        out.push_str("\n\n");
    }
    out
}

/// The REPL's default: just the answer, indented, then any footnote.
/// Errors, cards and anything without a result pod render as pods.
///
/// ```text
/// W› 5 mi in km
///   8.047 km
/// ```
pub fn render_compact(r: &Report, o: &Options) -> String {
    let answer = r
        .pods
        .iter()
        .find(|p| p.title == "result" || p.title == "defined");
    let Some(pod) = answer.filter(|_| r.ok) else {
        return render_pods(r, o);
    };
    let t = FILAMENT;
    let avail = o.width.saturating_sub(4).max(20);
    let mut out = String::new();
    // An assumption changes what the answer means; it stays, quietly.
    for p in r.pods.iter().filter(|p| p.title == "assuming") {
        for l in body(&p.body, o, avail) {
            out.push_str("  ");
            for (text, _) in &l {
                t.paint(Style::Dim, text, o.color, &mut out);
            }
            out.push('\n');
        }
    }
    for l in body(&pod.body, o, avail) {
        out.push_str("  ");
        for (text, style) in &l {
            t.paint(*style, text, o.color, &mut out);
        }
        while out.ends_with(' ') {
            out.pop();
        }
        out.push('\n');
    }
    if let Some(f) = &r.footnote {
        out.push_str("  ");
        t.paint(Style::Dim, f, o.color, &mut out);
        out.push('\n');
    }
    out.push('\n');
    out
}

/// The pods, each followed by a blank line, and the widest line's width.
fn pods(pods: &[Pod], o: &Options) -> (String, usize) {
    let t = FILAMENT;
    let (glyph, gutter) = if o.fancy { ("◆", "│") } else { ("*", "|") };
    let avail = o.width.saturating_sub(4).max(20);

    let rendered: Vec<(&Pod, Vec<Spans>)> =
        pods.iter().map(|p| (p, body(&p.body, o, avail))).collect();
    let content = rendered
        .iter()
        .flat_map(|(p, ls)| ls.iter().map(|l| width(l) + 2).chain([p.title.width() + 2]))
        .max()
        .unwrap_or(0);

    let mut out = String::new();
    for (p, lines) in &rendered {
        let title_style = if p.error {
            Style::ErrorTitle
        } else {
            Style::Title
        };
        out.push_str("  ");
        t.paint(title_style, glyph, o.color, &mut out);
        out.push(' ');
        t.paint(title_style, &p.title, o.color, &mut out);
        out.push('\n');
        for l in lines {
            out.push_str("  ");
            t.paint(Style::Dim, gutter, o.color, &mut out);
            out.push(' ');
            for (text, style) in l {
                t.paint(*style, text, o.color, &mut out);
            }
            // No trailing whitespace, ever.
            while out.ends_with(' ') {
                out.pop();
            }
            out.push('\n');
        }
        out.push('\n');
    }
    (out, content)
}

fn elapsed(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 10.0 {
        format!("{ms:.1}ms")
    } else {
        format!("{ms:.0}ms")
    }
}

/// The bare answer for `-q`, or `None` for an error.
pub fn render_quiet(r: &Report, o: &Options) -> Option<String> {
    Some(match r.quiet.as_ref()? {
        Quiet::Value(n) => quiet(*n, o.sig),
        Quiet::Parts(parts) => parts
            .iter()
            .map(|(n, u)| format!("{} {}", quiet(*n, o.sig), u.display(false)))
            .collect::<Vec<_>>()
            .join(" "),
        Quiet::Text(t) => t.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(q: &str, width: usize) -> String {
        let report = tungsten_pods::build(q, &tungsten_core::evaluate(q));
        let o = Options {
            width,
            color: false,
            fancy: true,
            sig: None,
            elapsed: None,
        };
        render(&report, &o).replace('\u{202F}', " ")
    }

    #[test]
    fn layout() {
        let out = run("60 mph * 2h 15min", 80);
        assert!(out.contains("  ◆ result\n  │ 135 mi\n"), "{out}");
        assert!(
            out.contains("217.3 km  ·  237 600 yd  ·  217 261 m"),
            "{out}"
        );
        assert!(out.trim_end().ends_with("W74"));
        assert!(!out.lines().any(|l| l.ends_with(' ')));
    }

    #[test]
    fn narrow_lists_stack() {
        let out = run("60 mph * 2h 15min", 40);
        assert!(out.contains("  │ 217.3 km\n  │ 237 600 yd\n"), "{out}");
    }

    #[test]
    fn per_and_plural_words() {
        assert!(run("3 a day", 80).contains("│ 3 per day"));
        assert!(run("2 cup", 80).contains("│ 2 cups"));
        assert!(run("1 cup", 80).contains("│ 1 cup\n"));
    }

    #[test]
    fn colour_only_when_asked() {
        let report = tungsten_pods::build("1 km", &tungsten_core::evaluate("1 km"));
        let o = Options {
            width: 80,
            color: true,
            fancy: true,
            sig: None,
            elapsed: None,
        };
        assert!(render(&report, &o).contains("\x1b["));
        let o = Options { color: false, ..o };
        assert!(!render(&report, &o).contains('\x1b'));
    }

    #[test]
    fn plain_is_ascii() {
        let report = tungsten_pods::build("3 m + 2 s", &tungsten_core::evaluate("3 m + 2 s"));
        let o = Options {
            width: 80,
            color: false,
            fancy: false,
            sig: None,
            elapsed: None,
        };
        assert!(render(&report, &o).is_ascii());
    }
}
