//! Scripted REPL sessions: each tests/sessions/NAME.txt is typed line by line
//! into the REPL, and the transcript is compared against
//! tests/snap/session-NAME.snap.

use std::path::{Path, PathBuf};
use tungsten::Settings;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn transcript(script: &str) -> String {
    tungsten::repl::transcript(
        script,
        Settings {
            width: 80,
            color: false,
            fancy: true,
            sig: None,
            timing: false,
            prefer: None,
            why: false,
        },
    )
}

#[test]
fn sessions() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("tests/sessions"))
        .expect("tests/sessions")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(root().join("tests/snap"));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings.bind(|| {
        for f in &files {
            let name = f.file_stem().and_then(|s| s.to_str()).expect("name");
            let script = std::fs::read_to_string(f).expect("script");
            insta::assert_snapshot!(format!("session-{name}"), transcript(&script));
        }
    });
}
