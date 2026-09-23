---
id: 51
title: Function properties pod
type: feature
status: backlog
milestone: v0.5
depends_on:
- 50
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: pods
effort: m
spec: §5
---

## Behaviour

For a plotted or defined function: zeros in range, parity, limits at
singularities, extrema.

## Acceptance criteria

- [ ] Snapshot test: `sin(x)/x` reports even, zeros at kπ, limit 1 at 0
