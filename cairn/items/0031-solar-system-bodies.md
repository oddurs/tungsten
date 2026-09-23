---
id: 31
title: Solar system bodies
type: data
status: done
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Every entry has a `source`
- [x] `w --why mars.gravity` shows provenance

## 2026-09-23

Done. Sun, 8 planets, the Moon, Pluto and 20 more major moons from NASA NSSDCA fact sheets (scripts/data/solar.py): mass, radius, density, gravity (gas giants at the 1-bar level, marked), escape velocity, orbital and rotation periods, distances, axial tilt, mean temperature. Mean radii of irregular moons are computed from the three published semi-axes and marked as derived in their source. '--why mars.gravity' snapshot.
