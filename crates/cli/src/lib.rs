//! The `tungsten` command. `main.rs` handles arguments and I/O; this is the
//! part the snapshot tests drive directly, so tests see exactly what users see.

use std::time::Instant;
use tungsten_render::Options;

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub width: usize,
    pub color: bool,
    pub fancy: bool,
    pub sig: Option<u32>,
    /// Show evaluation time in the footer. Off in tests.
    pub timing: bool,
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

/// Evaluates one query and renders its pods.
pub fn render_query(query: &str, s: &Settings) -> Rendered {
    let start = Instant::now();
    let result = tungsten_core::evaluate(query);
    let report = tungsten_pods::build(query, &result);
    let elapsed = s.timing.then(|| start.elapsed());
    let out = tungsten_render::render(&report, &options(s, elapsed));
    Rendered {
        out,
        err: String::new(),
        ok: report.ok,
    }
}

/// Evaluates one query and prints only its value: `8.04672`.
pub fn render_quiet(query: &str, s: &Settings) -> Rendered {
    let result = tungsten_core::evaluate(query);
    let report = tungsten_pods::build(query, &result);
    match tungsten_render::render_quiet(&report, &options(s, None)) {
        Some(v) => Rendered {
            out: format!("{v}\n"),
            err: String::new(),
            ok: true,
        },
        None => {
            let line = report.error_line.unwrap_or_else(|| "error".into());
            Rendered {
                out: String::new(),
                err: format!("tungsten: {line}\n"),
                ok: false,
            }
        }
    }
}
