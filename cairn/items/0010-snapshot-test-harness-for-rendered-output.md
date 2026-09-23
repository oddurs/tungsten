---
id: 10
title: Snapshot test harness for rendered output
type: chore
status: done
milestone: v0.1
depends_on:
- 9
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: infra
effort: s
spec: §12
---

The output *is* the product, so the primary test is: query in, rendered text
out, compared against a golden file. Use `insta`, render with colour off and a
fixed width of 80 so snapshots are stable.

Also add a `tests/queries.txt` corpus: one query per line, each producing a
snapshot. Adding a test should be adding a line.

## Acceptance criteria

- [x] `cargo insta test` runs every line of `tests/queries.txt`
- [x] Footer timing is masked in snapshots
- [x] A width-40 variant exists for the narrow layout

## 2026-09-22

Done. crates/cli/tests/snapshots.rs renders every line of tests/queries.txt at 80 and 40 columns into tests/snap/ (48 queries, 96 snapshots). `cargo insta test` runs them. Timing is masked by rendering with Settings.timing = false, which drops it from the footer entirely (`W74`). The same test keeps README examples equal to their snapshots.
