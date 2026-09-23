---
id: 40
title: REPL on reedline
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 39
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p0
area: repl
effort: m
spec: §8
---

## Behaviour

`w` with no arguments on a TTY opens the REPL. Prompt `W›`. Results render as
pods but more compactly: result pod only, `:pods on` for the full stack.
History persists at `~/.local/share/tungsten/history`.

## Acceptance criteria

- [x] Snapshot test via a scripted session
- [x] Ctrl-C clears the line, Ctrl-D exits

## 2026-09-23

reedline 0.49 (0.50+ needs rustc 1.95; workspace MSRV corrected to 1.88, which let-chains already required). Behaviour is a pure state machine (cli/src/repl.rs) driven by scripted sessions in tests/sessions/*.txt → tests/snap/session-*.snap; the terminal glue (cli/src/editor.rs) is tested in a real pty with rexpect: Ctrl-C clears, Ctrl-D exits, history survives a restart. Compact output = the answer alone (render_compact); cards, errors and assumptions keep their pods. History at $XDG_DATA_HOME/tungsten/history, synced after every line. Release binary 1.6 → 2.2 MB.
