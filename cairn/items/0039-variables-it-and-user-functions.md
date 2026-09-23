---
id: 39
title: Variables, it, and user functions
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 17
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p0
area: language
effort: m
spec: §2
---

## Behaviour

`x = expr` binds; `f(x) = expr` defines; `it` is the previous result. Bindings
are shared by the REPL and notebooks. Redefining a unit name is an error.

## Acceptance criteria

- [x] Snapshot test covers a three-line session using `it`
- [x] `m = 5` errors: `m` is metres
- [x] `a = 2; a * 3` is 6, even though `a` between operands means per (moved from 0013)

## 2026-09-23

Session in core: `name = expr` binds (and `it`), `f(x, y) = body` defines; bodies re-parse per call with params bound. Names that are units, built-ins, structural keywords or unshadowed things are refused; sugar keywords (`a`, `x`) can be bound. Recursive definitions (any call cycle) are refused since there are no conditionals; call depth capped at 16. Check memoizes user-call values so nested calls don't cost 2^depth. CLI: `;` splits statements; stdin lines share a session.
