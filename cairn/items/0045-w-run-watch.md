---
id: 45
title: w run --watch
type: feature
status: backlog
milestone: v0.4
depends_on:
- 44
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: notebook
effort: s
spec: §9
---

## Behaviour

Re-evaluate on save and redraw in place. Built for a split pane next to an
editor.

## Acceptance criteria

- [ ] Redraw has no flicker (full frame written in one syscall)
