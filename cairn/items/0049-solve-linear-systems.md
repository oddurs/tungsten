---
id: 49
title: 'solve: linear systems'
type: feature
status: backlog
milestone: v0.5
depends_on:
- 17
created: 2026-09-22
updated: 2026-09-22
priority: p2
area: language
effort: m
spec: §5
---

## Behaviour

`solve 2x + y = 5, x - y = 1` via exact rational Gaussian elimination.

## Acceptance criteria

- [ ] Snapshot test for a 2×2 and a 3×3 system
- [ ] Singular systems say "no unique solution"
