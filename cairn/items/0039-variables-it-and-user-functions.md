---
id: 39
title: Variables, it, and user functions
type: feature
status: backlog
milestone: v0.3
depends_on:
- 17
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: language
effort: m
spec: §2
---

## Behaviour

`x = expr` binds; `f(x) = expr` defines; `it` is the previous result. Bindings
are shared by the REPL and notebooks. Redefining a unit name is an error.

## Acceptance criteria

- [ ] Snapshot test covers a three-line session using `it`
- [ ] `m = 5` errors: `m` is metres
- [ ] `a = 2; a * 3` is 6, even though `a` between operands means per (moved from 0013)
