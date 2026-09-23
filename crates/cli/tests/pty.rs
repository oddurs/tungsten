//! The REPL in a real terminal: keys, signals and history.

#![cfg(unix)]

use rexpect::ReadUntil;
use rexpect::session::{PtySession, spawn_command};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Before drawing a prompt, reedline asks the terminal where the cursor is
/// (`ESC [6n`). As the terminal here, the test answers every time.
const CURSOR_QUERY: &str = "\x1b[6n";

fn spawn(data: &Path) -> PtySession {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tungsten"));
    // Plain: the pty is read as Latin-1, so keep output ASCII.
    cmd.arg("--plain")
        .env("XDG_DATA_HOME", data)
        .env("TERM", "xterm-256color")
        .env_remove("NO_COLOR");
    let mut p = spawn_command(cmd, Some(20_000)).expect("spawn tungsten");
    expect(&mut p, "W> ");
    p
}

/// Waits for `needle`, answering cursor queries on the way.
fn expect(p: &mut PtySession, needle: &str) {
    loop {
        let (_, found) = p
            .exp_any(vec![
                ReadUntil::String(CURSOR_QUERY.into()),
                ReadUntil::String(needle.into()),
            ])
            .unwrap_or_else(|e| panic!("waiting for {needle:?}: {e}"));
        if found == CURSOR_QUERY {
            answer(p);
        } else {
            return;
        }
    }
}

fn answer(p: &mut PtySession) {
    keys(p, "\x1b[1;1R");
}

/// Types keys, flushed: rexpect only flushes by itself at a newline.
fn keys(p: &mut PtySession, s: &str) {
    p.send(s).expect("type");
    p.flush().expect("flush");
}

fn exits(p: &mut PtySession) {
    loop {
        match p.exp_any(vec![ReadUntil::String(CURSOR_QUERY.into()), ReadUntil::EOF]) {
            Ok((_, found)) if found == CURSOR_QUERY => {
                answer(p);
            }
            Ok(_) => return,
            Err(e) => panic!("waiting for exit: {e}"),
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tungsten-pty-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn ctrl_c_clears_the_line_and_ctrl_d_exits() {
    let dir = scratch("keys");
    let mut p = spawn(&dir);
    // Half a line, abandoned: had it survived, the next line would be
    // `5 km in blorp2 + 2`, an unknown word.
    keys(&mut p, "5 km in blorp");
    expect(&mut p, "blorp");
    p.send_control('c').expect("ctrl-c");
    expect(&mut p, "W> ");
    keys(&mut p, "2 + 2\r");
    expect(&mut p, "  4");
    p.send_control('d').expect("ctrl-d");
    exits(&mut p);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn history_survives_a_restart() {
    let dir = scratch("history");
    let mut p = spawn(&dir);
    keys(&mut p, "rent = 2400 USD/month\r");
    expect(&mut p, "  2400 USD/mo");
    keys(&mut p, ":quit\r");
    exits(&mut p);

    let history = std::fs::read_to_string(dir.join("tungsten/history")).expect("history file");
    assert!(history.contains("rent = 2400 USD/month"), "{history:?}");

    // A new session: up-arrow recalls the definition (before `:quit`), and
    // running it again brings `rent` back.
    let mut p = spawn(&dir);
    keys(&mut p, "\x1b[A");
    expect(&mut p, ":quit");
    keys(&mut p, "\x1b[A");
    expect(&mut p, "rent = 2400 USD/month");
    keys(&mut p, "\r");
    expect(&mut p, "  2400 USD/mo");
    keys(&mut p, "rent / 4\r");
    expect(&mut p, "  600 USD/mo");
    p.send_control('d').expect("ctrl-d");
    exits(&mut p);
    let _ = std::fs::remove_dir_all(&dir);
}
