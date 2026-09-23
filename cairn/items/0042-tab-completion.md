---
id: 42
title: Tab completion
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
effort: s
spec: §8
---

## Behaviour

Complete units, entities, properties after a possessive (`earth's <tab>`),
variables and meta-commands.

## Acceptance criteria

- [x] Completion after `earth's` lists only earth's properties

## 2026-09-23

core::complete(line, pos, scope): after 'earth\'s ' or 'earth.' only that thing's properties and facts; otherwise session names first, then functions, units (symbols case-exact, names case-free), things, properties; at most 40. ':' completes meta-commands.
