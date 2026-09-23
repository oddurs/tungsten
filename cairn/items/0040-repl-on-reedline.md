---
id: 40
title: REPL on reedline
type: feature
status: backlog
milestone: v0.3
depends_on:
- 39
created: 2026-09-22
updated: 2026-09-22
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

- [ ] Snapshot test via a scripted session
- [ ] Ctrl-C clears the line, Ctrl-D exits
