---
id: 31
title: Solar system bodies
type: data
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p2
area: kb
effort: s
spec: §4
---

## What to add

Sun, 8 planets, Pluto, the major moons: mass, radius, surface gravity, orbital
period, mean distance from the Sun.

## Sources

NASA planetary fact sheets (nssdc.gsfc.nasa.gov/planetary/factsheet).

## Acceptance criteria

- [ ] Every entry has a `source`
- [ ] `w --why mars.gravity` shows provenance
