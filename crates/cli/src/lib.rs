//! The `tungsten` command. `main.rs` handles arguments and I/O; this is the
//! part the snapshot tests drive directly, so tests see exactly what users see.

use std::time::Instant;
use tungsten_core::Session;
use tungsten_render::Options;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub width: usize,
    pub color: bool,
    pub fancy: bool,
    pub sig: Option<u32>,
    /// Show evaluation time in the footer. Off in tests.
    pub timing: bool,
    /// `--as element`: what an ambiguous name should mean.
    pub prefer: Option<tungsten_core::Kind>,
    /// `--why`: list where every unit and value came from.
    pub why: bool,
}

impl Settings {
    /// A new session that evaluates with these settings.
    pub fn session(&self) -> Session {
        Session::new(tungsten_core::Options {
            prefer: self.prefer,
        })
    }
}

/// `rent = 2400 USD/month; rent * 12 month`: statements, in order.
pub fn statements(line: &str) -> impl Iterator<Item = &str> {
    line.split(';').map(str::trim).filter(|s| !s.is_empty())
}

#[derive(Debug)]
pub struct Rendered {
    /// Text for stdout.
    pub out: String,
    /// Text for stderr (only in quiet mode, for errors).
    pub err: String,
    pub ok: bool,
}

fn options(s: &Settings, elapsed: Option<std::time::Duration>) -> Options {
    Options {
        width: s.width,
        color: s.color,
        fancy: s.fancy,
        sig: s.sig,
        elapsed,
    }
}

/// Evaluates one query, in a session of its own, and renders its pods.
pub fn render_query(query: &str, s: &Settings) -> Rendered {
    render_line(query, &mut s.session(), s)
}

/// Evaluates a line of `;`-separated statements in `session` and renders
/// each one's pods.
pub fn render_line(line: &str, session: &mut Session, s: &Settings) -> Rendered {
    let mut all = Rendered {
        out: String::new(),
        err: String::new(),
        ok: true,
    };
    let mut any = false;
    for stmt in statements(line) {
        any = true;
        let start = Instant::now();
        let result = session.run(stmt);
        let report = tungsten_pods::build_with(stmt, &result, tungsten_pods::Build { why: s.why });
        let elapsed = s.timing.then(|| start.elapsed());
        all.out
            .push_str(&tungsten_render::render(&report, &options(s, elapsed)));
        all.ok &= report.ok;
    }
    if !any {
        // An empty query still explains itself.
        let result = session.run(line);
        let report = tungsten_pods::build_with(line, &result, tungsten_pods::Build::default());
        all.out = tungsten_render::render(&report, &options(s, None));
        all.ok = report.ok;
    }
    all
}

/// Evaluates one query and prints only its value: `8.04672`.
pub fn render_quiet(query: &str, s: &Settings) -> Rendered {
    quiet_line(query, &mut s.session(), s)
}

/// Evaluates a line of statements in `session`, printing only values.
pub fn quiet_line(line: &str, session: &mut Session, s: &Settings) -> Rendered {
    let mut all = Rendered {
        out: String::new(),
        err: String::new(),
        ok: true,
    };
    for stmt in statements(line) {
        let result = session.run(stmt);
        let report = tungsten_pods::build_with(stmt, &result, tungsten_pods::Build { why: s.why });
        match tungsten_render::render_quiet(&report, &options(s, None)) {
            Some(v) => all.out.push_str(&format!("{v}\n")),
            None => {
                let line = report.error_line.unwrap_or_else(|| "error".into());
                all.err.push_str(&format!("tungsten: {line}\n"));
                all.ok = false;
            }
        }
    }
    all
}

/// `tungsten --about`: the element card as the version screen.
pub fn render_about(s: &Settings) -> Rendered {
    let report = tungsten_pods::about(env!("CARGO_PKG_VERSION"));
    Rendered {
        out: tungsten_render::render(&report, &options(s, None)),
        err: String::new(),
        ok: true,
    }
}
