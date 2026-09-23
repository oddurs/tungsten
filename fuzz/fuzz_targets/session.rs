//! A session: `;`-separated statements (assignments, definitions, `it`) run
//! in order, each highlighted and completed at every position, as the REPL
//! would. Nothing may panic, and highlighting must stay within the line.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else { return };
    let mut s = tungsten_core::Session::default();
    for stmt in src.split(';').take(8) {
        let scope = s.scope();
        for (span, _) in tungsten_core::classify(stmt, &scope) {
            assert!(span.start <= span.end && span.end <= stmt.len());
            assert!(stmt.is_char_boundary(span.start) && stmt.is_char_boundary(span.end));
        }
        for (pos, _) in stmt.char_indices().step_by(3) {
            let c = tungsten_core::complete(stmt, pos, &scope);
            assert!(c.span.end == pos && stmt.is_char_boundary(c.span.start));
        }
        let result = s.run(stmt);
        let report = tungsten_pods::build(stmt, &result);
        let o = tungsten_render::Options { width: 60, color: false, fancy: true, sig: None, elapsed: None };
        let _ = tungsten_render::render_compact(&report, &o);
    }
});
