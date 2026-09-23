---
id: 28
title: Knowledge-base build pipeline
type: feature
status: backlog
milestone: v0.2
depends_on:
- 15
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: kb
effort: m
spec: §4
---

## Behaviour

`build.rs` reads `data/*.toml`, validates every quantity string through the
unit parser, and emits `phf` maps. Bad data fails the build, not a query.

## Acceptance criteria

- [ ] Snapshot test covers `w --why earth.mass`
- [ ] A typo'd unit in any data file is a build error naming file and line
- [ ] Binary size stays under 5 MB with all v0.2 data
