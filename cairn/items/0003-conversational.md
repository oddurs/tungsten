---
id: 3
key: v0.3
title: Conversational
type: milestone
status: done
assignee: Oddur Sigurdsson
depends_on:
- 2
- 64
- 68
- 71
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
---

`w` with no arguments becomes a REPL: variables, `it`, user functions, highlighting as you type, completion.

## Acceptance criteria

- [x] A session can define `rent`, use `it`, and survive a restart via history

## 2026-09-23

Shipped as 0.3.0 (#22, #23, this PR). The criterion is tested end to end in crates/cli/tests/pty.rs::history_survives_a_restart: define rent, quit, restart, recall it with up-arrow, use it. Fuzzed 3 min each: query (280k runs) and a new session target (assignments, definitions, highlighting and completion at every position; 36k runs), no findings.
