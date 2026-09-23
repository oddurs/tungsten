---
id: 41
title: Live syntax highlighting
type: feature
status: backlog
milestone: v0.3
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: repl
effort: m
spec: §8
---

## Behaviour

Colour tokens as typed: numbers amber, units cyan, entities green, variables
white, unknown words underlined. Uses the same resolver as evaluation, so what
is highlighted is what will be understood.

## Acceptance criteria

- [ ] Highlighter unit-tested against resolver output
