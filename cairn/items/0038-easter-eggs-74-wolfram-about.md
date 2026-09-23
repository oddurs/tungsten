---
id: 38
title: 'Easter eggs: 74, wolfram, --about'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p3
area: cli
effort: s
spec: §11
---

## Behaviour

- `w 74` answers normally, then a dim line: `74 is tungsten's atomic number.`
- `w wolfram` shows tungsten's element card with the *Wolf Rahm* etymology.
- `w --about` is the element card as the version screen.

## Acceptance criteria

- [ ] Snapshot test covers all three
