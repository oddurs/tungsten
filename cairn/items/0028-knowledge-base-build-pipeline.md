---
id: 28
title: Knowledge-base build pipeline
type: feature
status: doing
milestone: v0.2
assignee: Oddur Sigurdsson
claimed: 2026-09-23
depends_on:
- 15
created: 2026-09-22
updated: 2026-09-23
priority: p0
area: kb
effort: m
spec: §4
---

## Behaviour

`build.rs` reads `data/*.toml`, validates every quantity string through the
unit parser, and emits `phf` maps. Bad data fails the build, not a query.

## Acceptance criteria

- [ ] Snapshot test covers `w --why earth.mass`
- [ ] A typo'd unit in any data file is a build error naming file and line
- [ ] Binary size stays under 5 MB with all v0.2 data

## 2026-09-23

Pipeline landed (PR: Compile the knowledge base). crates/kb/build.rs reads data/{properties,constants,elements,solar,items}.toml, parses every value with the unit parser, checks it against its property's declared dimension, and fails the build with file:line. Verified by planting errors: 'data/solar.toml:76: Earth mass = "5.9722e24 kgg": unknown unit "kgg"' and '… "5.9722e24 km" is length, but mass is mass'. Deviation from the text above: sorted static arrays with binary search instead of phf, the same approach as the unit table, so no new dependency. Still open: the --why snapshot (needs 0033 and 0037) and the binary-size check, which happens once the CLI links kb.
