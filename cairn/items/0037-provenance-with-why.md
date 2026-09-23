---
id: 37
title: Provenance with --why
type: feature
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p2
area: cli
effort: s
spec: §4
---

## Behaviour

`--why` appends a sources pod listing the provenance of every knowledge-base
value the query touched.

## Acceptance criteria

- [ ] Snapshot test covers `w --why 3 coffees in mg of caffeine`
