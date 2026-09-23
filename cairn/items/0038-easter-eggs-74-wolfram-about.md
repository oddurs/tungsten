---
id: 38
title: 'Easter eggs: 74, wolfram, --about'
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Snapshot test covers all three

## 2026-09-23

Done, all three in snapshots. '74' (and anything that comes to exactly 74, like '70 + 4') answers normally, then a grey line: '74 is tungsten's atomic number.' 'wolfram' is an alias of tungsten, so it shows the element card with the sourced etymology. '--about' (hidden from --help) shows the tungsten card, then the version, home, data sources and licence. The workspace version is now 0.2.0.
