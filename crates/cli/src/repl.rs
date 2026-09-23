//! The REPL's behaviour, apart from the terminal: a line in, text out. The
//! snapshot tests drive this directly with scripted sessions.
//!
//! ```text
//! W› rent = 2400 USD/month
//!   2 400 USD/mo
//!
//! W› :vars
//!   ◆ variables
//!   │ it    = 2 400 USD/mo
//!   │ rent  = 2 400 USD/mo
//! ```

use crate::{Settings, options, statements};
use std::time::Instant;
use tungsten_core::{Outcome, Scope, Session};
use tungsten_pods::{Body, Build, Line, NumMode, Pod, Report, Seg};
use tungsten_render::{FILAMENT, Style};

/// The colon commands, for `:help` and completion.
pub const COMMANDS: &[(&str, &str)] = &[
    (":vars", "variables and functions defined so far"),
    (":clear", "forget them all"),
    (":pods on|off", "every pod, or just the answer"),
    (":sig N", "significant figures (:sig alone resets to 4)"),
    (":why", "where the last answer's units and values came from"),
    (":help", "this list"),
    (":quit", "leave (or Ctrl-D)"),
];

#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    /// Print this (possibly nothing) and read another line.
    Show(String),
    Quit,
}

#[derive(Debug)]
pub struct Repl {
    session: Session,
    settings: Settings,
    /// `:pods on`: the full stack of pods, not just the answer.
    pods: bool,
    /// The last statement that succeeded, for `:why`.
    last: Option<(String, Outcome)>,
}

impl Repl {
    pub fn new(settings: Settings) -> Self {
        Self {
            session: settings.session(),
            settings,
            pods: false,
            last: None,
        }
    }

    /// Names the session has bound, for highlighting and completion.
    pub fn scope(&self) -> Scope {
        self.session.scope()
    }

    /// The terminal may be resized between lines.
    pub fn set_width(&mut self, width: usize) {
        self.settings.width = width;
    }

    pub fn line(&mut self, line: &str) -> Reply {
        let line = line.trim();
        if line.is_empty() {
            return Reply::Show(String::new());
        }
        if let Some(cmd) = line.strip_prefix(':') {
            return self.command(cmd.trim());
        }
        let mut out = String::new();
        for stmt in statements(line) {
            let start = Instant::now();
            let result = self.session.run(stmt);
            let report = tungsten_pods::build_with(stmt, &result, Build::default());
            if let Ok(o) = result {
                self.last = Some((stmt.to_string(), o));
            }
            if self.pods {
                let elapsed = self.settings.timing.then(|| start.elapsed());
                out.push_str(&tungsten_render::render_pods(
                    &report,
                    &options(&self.settings, elapsed),
                ));
            } else {
                out.push_str(&tungsten_render::render_compact(
                    &report,
                    &options(&self.settings, None),
                ));
            }
        }
        Reply::Show(out)
    }

    fn command(&mut self, cmd: &str) -> Reply {
        let (name, arg) = match cmd.split_once(char::is_whitespace) {
            Some((n, a)) => (n, a.trim()),
            None => (cmd, ""),
        };
        let shown = match (name, arg) {
            ("quit" | "q" | "exit", "") => return Reply::Quit,
            ("help" | "h" | "?", "") => self.pods_out(help()),
            ("vars", "") => self.pods_out(self.vars()),
            ("clear", "") => {
                self.session.clear();
                self.last = None;
                self.note("cleared")
            }
            ("pods", "") => {
                self.pods = !self.pods;
                self.note(if self.pods { "pods on" } else { "pods off" })
            }
            ("pods", "on") => {
                self.pods = true;
                self.note("pods on")
            }
            ("pods", "off") => {
                self.pods = false;
                self.note("pods off")
            }
            ("sig", "") => {
                self.settings.sig = None;
                self.note("4 significant figures")
            }
            ("sig", n) => match n.parse::<u32>() {
                Ok(n @ 1..=17) => {
                    self.settings.sig = Some(n);
                    let s = if n == 1 { "" } else { "s" };
                    self.note(&format!("{n} significant figure{s}"))
                }
                _ => self.pods_out(problem(
                    "can't set that",
                    &format!(":sig {n}"),
                    "significant figures are a whole number from 1 to 17",
                )),
            },
            ("why", "") => self.why(),
            _ => self.pods_out(problem(
                "unknown command",
                &format!(":{cmd}"),
                "not a command; try :help",
            )),
        };
        Reply::Show(shown)
    }

    fn pods_out(&self, r: Report) -> String {
        tungsten_render::render_pods(&r, &options(&self.settings, None))
    }

    /// `  pods on`: a quiet confirmation.
    fn note(&self, text: &str) -> String {
        let mut out = String::from("  ");
        FILAMENT.paint(Style::Dim, text, self.settings.color, &mut out);
        out.push_str("\n\n");
        out
    }

    fn vars(&self) -> Report {
        let vars = self.session.vars();
        let funcs = self.session.funcs();
        if vars.is_empty() && funcs.is_empty() {
            return report(
                "variables",
                vec![Line(vec![Seg::Dim(
                    "nothing defined yet; try rent = 2400 USD/month".into(),
                )])],
            );
        }
        // `it` last: it is not something the user named.
        let names = vars
            .keys()
            .filter(|n| *n != "it")
            .chain(vars.keys().filter(|n| *n == "it"));
        let pad = vars.keys().map(|n| n.chars().count()).max().unwrap_or(0);
        let mut lines: Vec<Line> = names
            .map(|name| {
                let v = &vars[name];
                let mut l = Line(vec![Seg::Text(format!("{name:<pad$} = "))]);
                match v.shown() {
                    Ok(num) => l.push(Seg::Value {
                        num,
                        unit: v.unit.clone(),
                        mode: NumMode::Result,
                    }),
                    Err(_) => l.push(Seg::Dim("(too large to show)".into())),
                };
                l
            })
            .collect();
        for (name, f) in funcs {
            lines.push(Line(vec![Seg::Text(format!(
                "{name}({}) = {}",
                f.params.join(", "),
                f.body
            ))]));
        }
        report("variables", lines)
    }

    fn why(&self) -> String {
        let Some((input, outcome)) = &self.last else {
            return self.note("nothing calculated yet");
        };
        let full = tungsten_pods::build_with(input, &Ok(outcome.clone()), Build { why: true });
        let sources: Vec<Pod> = full
            .pods
            .into_iter()
            .filter(|p| p.title == "sources")
            .collect();
        if sources.is_empty() {
            return self.note(&format!("{input} used no units or data"));
        }
        self.pods_out(Report {
            pods: sources,
            ok: true,
            quiet: None,
            error_line: None,
            footnote: None,
        })
    }
}

/// A scripted session as it would appear on screen: each line after the
/// prompt, then its output. Lines starting with `#` are comments.
pub fn transcript(script: &str, settings: Settings) -> String {
    let mut repl = Repl::new(settings);
    let mut out = String::new();
    for line in script.lines().filter(|l| !l.starts_with('#')) {
        out.push_str(&format!("W› {line}\n"));
        match repl.line(line) {
            Reply::Show(text) => out.push_str(&text),
            Reply::Quit => {
                out.push_str("(exit)\n");
                break;
            }
        }
    }
    out.trim_end().to_string()
}

fn report(title: &str, lines: Vec<Line>) -> Report {
    Report {
        pods: vec![Pod {
            title: title.into(),
            error: false,
            body: Body::Lines(lines),
        }],
        ok: true,
        quiet: None,
        error_line: None,
        footnote: None,
    }
}

fn help() -> Report {
    let pad = COMMANDS.iter().map(|(c, _)| c.len()).max().unwrap_or(0);
    let mut lines: Vec<Line> = COMMANDS
        .iter()
        .map(|(c, what)| {
            Line(vec![
                Seg::Text(format!("{c:<pad$}  ")),
                Seg::Dim((*what).into()),
            ])
        })
        .collect();
    lines.push(Line(vec![Seg::Dim(
        "Tab completes; Ctrl-C clears the line; history is kept between sessions".into(),
    )]));
    report("commands", lines)
}

fn problem(title: &str, input: &str, note: &str) -> Report {
    Report {
        pods: vec![Pod {
            title: title.into(),
            error: true,
            body: Body::Pre(vec![
                Line(vec![Seg::Text(input.into())]),
                Line(vec![Seg::Dim(note.into())]),
            ]),
        }],
        ok: false,
        quiet: None,
        error_line: None,
        footnote: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repl() -> Repl {
        Repl::new(Settings {
            width: 80,
            color: false,
            fancy: true,
            sig: None,
            timing: false,
            prefer: None,
            why: false,
        })
    }

    fn show(r: Reply) -> String {
        match r {
            Reply::Show(s) => s,
            Reply::Quit => "(quit)".into(),
        }
    }

    #[test]
    fn quit_and_empty_lines() {
        let mut r = repl();
        assert_eq!(r.line("   "), Reply::Show(String::new()));
        assert_eq!(r.line(":q"), Reply::Quit);
        assert_eq!(r.line(":quit"), Reply::Quit);
    }

    #[test]
    fn sig_is_bounded() {
        let mut r = repl();
        assert!(show(r.line(":sig 0")).contains("can't set that"));
        assert!(show(r.line(":sig 99")).contains("1 to 17"));
        assert!(show(r.line(":sig 6")).contains("6 significant figures"));
        assert!(show(r.line("1/7 + 0.0")).contains("0.142857"));
    }
}
