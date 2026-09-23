//! Any string, all the way through: lex, parse, check, evaluate, build pods,
//! render. Nothing may panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(q) = std::str::from_utf8(data) else { return };
    let result = tungsten_core::evaluate(q);
    let report = tungsten_pods::build(q, &result);
    for (width, fancy) in [(80, true), (40, false)] {
        let o = tungsten_render::Options { width, color: true, fancy, sig: None, elapsed: None };
        let _ = tungsten_render::render(&report, &o);
        let _ = tungsten_render::render_quiet(&report, &o);
    }
});
