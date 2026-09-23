---
id: 37
title: Provenance with --why
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
area: cli
effort: s
spec: §4
---

## Behaviour

`--why` appends a sources pod listing the provenance of every knowledge-base
value the query touched.

## Acceptance criteria

- [x] Snapshot test covers `w --why 3 coffees in mg of caffeine`

## 2026-09-23

Done. --why appends a sources pod listing every unit and knowledge-base value the query touched, in order, deduplicated. Unit citation codes from units.toml expand to full titles (HB44 → NIST Handbook 44 (2024), Appendix C). A card lists the entity's source plus any value or fact with its own. Snapshots: --why 3 coffees in mg of caffeine, earth.mass, gold.density, mars.gravity, c, tungsten, 60 mph × 2h 15min. The corpus now accepts leading --why/--as flags. Found while testing: tungsten's etymology note was attributed to PubChem, which says nothing about it. It now carries its own source (RSC for 'tung sten', Wikipedia for wolf rahm and lupi spuma), and the text is corrected: wolf rahm is 'wolf's cream'; 'wolf's froth' is Agricola's Latin.
