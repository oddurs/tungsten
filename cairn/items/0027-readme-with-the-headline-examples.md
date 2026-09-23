---
id: 27
title: README with the headline examples
type: docs
status: done
milestone: v0.1
depends_on:
- 21
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
area: docs
effort: s
---

The README leads with three queries and their output, pasted from snapshots so
it can never be wrong.

## Acceptance criteria

- [x] README examples are generated from the snapshot files
- [x] Links to docs/concept.md and ROADMAP.md

## 2026-09-22

Done. README.md leads with three examples wrapped in <!-- snap: … --> markers. crates/cli/tests/snapshots.rs::readme_matches_snapshots fails if any block differs from its snapshot, and `UPDATE_README=1 cargo test` rewrites them. Links docs/concept.md and ROADMAP.md, and documents install, the w alias, options and exit codes.
