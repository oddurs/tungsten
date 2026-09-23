use clap::{ArgAction, Parser, ValueEnum};
use std::io::{BufRead, IsTerminal, Write};
use std::process::ExitCode;
use tungsten::{Settings, quiet_line, render_line};
use tungsten_core::Kind;

fn parse_kind(s: &str) -> Result<Kind, String> {
    Kind::parse(s).ok_or_else(|| {
        let all: Vec<String> = Kind::ALL
            .iter()
            .map(|k| k.name().replace(' ', "-"))
            .collect();
        format!("expected one of: {}", all.join(", "))
    })
}

const HELP: &str = "\
{name} {version} — calculate with quantities and units

Usage: tungsten [OPTIONS] <QUERY>...
       echo \"3 ft in cm\" | tungsten -q
       tungsten                      (interactive; :help inside)

Examples:
  tungsten 60 mph x 2h 15min in km
  tungsten 1/3 + 1/6
  tungsten \"5'11\\\" in cm\"

Options:
{options}

Shells treat * ? ' \" ( ) specially. Quote the query, or write
x for *, per for /, and ft/in for ' and \".
Tip: alias w=tungsten  (fish: abbr -a w tungsten)";

#[derive(Clone, Copy, Debug, ValueEnum)]
enum When {
    Auto,
    Always,
    Never,
}

#[derive(Parser, Debug)]
#[command(
    name = "tungsten",
    version,
    about,
    help_template = HELP,
    allow_negative_numbers = true,
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Args {
    /// Print only the value, for scripts
    #[arg(short, long)]
    quiet: bool,

    /// ASCII output, no colour
    #[arg(long)]
    plain: bool,

    /// Significant figures [default: 4]
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u32).range(1..=17))]
    sig: Option<u32>,

    /// Show where every unit and value came from
    #[arg(long)]
    why: bool,

    /// About tungsten
    #[arg(long, hide = true)]
    about: bool,

    /// What an ambiguous name means: planet, element, constant, …
    #[arg(long = "as", value_name = "KIND", value_parser = parse_kind)]
    prefer: Option<Kind>,

    /// Colour: auto, always or never
    #[arg(
        long,
        value_name = "WHEN",
        default_value = "auto",
        hide_default_value = true
    )]
    color: When,

    /// Layout width in columns (default: the terminal's)
    #[arg(long, hide = true)]
    width: Option<usize>,

    /// Print help
    #[arg(short, long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,

    /// The query, in words and math
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    query: Vec<String>,
}

fn terminal_width() -> usize {
    if let Some((w, _)) = terminal_size::terminal_size() {
        return w.0 as usize;
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(80)
}

fn main() -> ExitCode {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("tungsten: internal error: {info}");
        eprintln!("This is a bug. Please report it with the query that caused it.");
        std::process::exit(1);
    }));

    let args = Args::parse();
    let stdout = std::io::stdout();
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let color = !args.plain
        && match args.color {
            When::Always => true,
            When::Never => false,
            When::Auto => stdout.is_terminal() && !no_color,
        };
    let settings = Settings {
        width: args.width.unwrap_or_else(terminal_width),
        color,
        fancy: !args.plain,
        sig: args.sig,
        timing: true,
        prefer: args.prefer,
        why: args.why,
    };

    if args.about {
        let r = tungsten::render_about(&settings);
        print!("{}", r.out);
        return ExitCode::SUCCESS;
    }

    let queries: Vec<String> = if !args.query.is_empty() {
        vec![args.query.join(" ")]
    } else if !std::io::stdin().is_terminal() {
        std::io::stdin()
            .lock()
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
            .collect()
    } else {
        let width = move || args.width.unwrap_or_else(terminal_width);
        return match tungsten::editor::run(settings, width) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("tungsten: {e}");
                ExitCode::from(1)
            }
        };
    };

    // Lines read from stdin share one session: `rent = …` then `rent * 12`.
    let mut session = settings.session();
    let mut ok = true;
    let mut out = stdout.lock();
    for q in &queries {
        let r = if args.quiet {
            quiet_line(q, &mut session, &settings)
        } else {
            render_line(q, &mut session, &settings)
        };
        ok &= r.ok;
        let _ = out.write_all(r.out.as_bytes());
        if !r.err.is_empty() {
            let _ = out.flush();
            eprint!("{}", r.err);
        }
    }
    let _ = out.flush();
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}
