//! The real binary: arguments, stdin, exit codes, quiet mode.

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str], stdin: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tungsten"));
    cmd.args(args)
        .env_remove("NO_COLOR")
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn tungsten");
    if let Some(input) = stdin {
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write");
    }
    child.wait_with_output().expect("wait")
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

#[test]
fn quiet_from_stdin() {
    // echo "3 ft in cm" | w -q
    let out = run(&["-q"], Some("3 ft in cm\n"));
    insta::assert_snapshot!(text(&out.stdout), @"91.44");
    assert!(out.status.success());
}

#[test]
fn quiet_from_arguments() {
    let out = run(&["-q", "5", "mi", "in", "km"], None);
    assert_eq!(text(&out.stdout), "8.04672\n");
}

#[test]
fn stdin_skips_blank_lines_and_comments() {
    let out = run(&["-q"], Some("# a notebook\n\n1 + 1\n2 h in min\n"));
    assert_eq!(text(&out.stdout), "2\n120\n");
}

#[test]
fn query_errors_exit_2() {
    let out = run(&["-q", "3", "m", "+", "2", "s"], None);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert_eq!(text(&out.stderr), "tungsten: can't add length and time\n");

    let out = run(&["3", "m", "+", "2", "s"], None);
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stdout).contains("can't add these"));
}

#[test]
fn one_bad_line_fails_the_run_but_not_the_others() {
    let out = run(&["-q"], Some("1 + 1\n3 m + 2 s\n2 + 2\n"));
    assert_eq!(text(&out.stdout), "2\n4\n");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn negative_numbers_are_not_flags() {
    let out = run(&["-q", "-40", "°C", "in", "°F"], None);
    assert_eq!(text(&out.stdout), "-40\n");
}

#[test]
fn help_fits_a_terminal() {
    let out = run(&["--help"], None);
    let help = text(&out.stdout);
    assert!(help.lines().count() <= 24, "{} lines", help.lines().count());
    assert!(help.contains("Shells treat * ? ' \" ( ) specially"));
}

#[test]
fn no_colour_when_piped_and_plain_is_ascii() {
    let out = run(&["60", "mph", "x", "2h", "15min"], None);
    let s = text(&out.stdout);
    assert!(!s.contains('\x1b'));
    assert!(s.contains('◆'));

    let out = run(
        &[
            "--plain", "--color", "always", "60", "mph", "x", "2h", "15min",
        ],
        None,
    );
    assert!(text(&out.stdout).is_ascii());
}

#[test]
fn colour_when_forced() {
    let out = run(&["--color", "always", "1", "km"], None);
    assert!(text(&out.stdout).contains("\x1b["));
}

#[test]
fn as_picks_the_kind() {
    let out = text(&run(&["--color", "never", "--as", "element", "mercury"], None).stdout);
    assert!(out.contains("mercury · Hg · 80"), "{out}");
    assert!(
        !out.contains("assuming"),
        "asked for, so nothing was assumed"
    );

    // W is the watt, until you ask for the element.
    let out = text(&run(&["--color", "never", "--as", "element", "W"], None).stdout);
    assert!(out.contains("tungsten · W · 74"), "{out}");

    // e is Euler's number, until you ask for the constant.
    let out = run(&["-q", "--as", "constant", "e"], None);
    assert_eq!(text(&out.stdout), "1.602176634e-19\n");
    let out = run(&["-q", "e"], None);
    assert_eq!(text(&out.stdout), "2.718281828459045\n");
}

#[test]
fn as_rejects_unknown_kinds() {
    let out = run(&["--as", "vegetable", "1"], None);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("expected one of: star, planet"));
}

#[test]
fn quiet_cards() {
    assert_eq!(text(&run(&["-q", "G"], None).stdout), "6.6743e-11\n");
    let out = run(&["-q", "gold"], None);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        text(&out.stderr),
        "tungsten: gold is a thing, not a quantity\n"
    );
}
