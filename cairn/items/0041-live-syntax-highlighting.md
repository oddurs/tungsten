---
id: 41
title: Live syntax highlighting
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Highlighter unit-tested against resolver output

## 2026-09-23

core::classify(src, scope) runs the lexer and a lenient resolver (unknown words recorded, not fatal), so highlighting shows exactly what evaluation reads. Numbers amber 136, units teal 30, things/properties green 64, variables bold, unknown underlined. Tested in core against evaluate() and in the editor adapter span-for-span.
