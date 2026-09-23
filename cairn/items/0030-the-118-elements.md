---
id: 30
title: The 118 elements
type: data
status: done
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: kb
effort: m
spec: §4
---

## What to add

Symbol, name, atomic number, standard atomic weight, density, melting and
boiling points, and etymology (needed by the `w wolfram` easter egg).

## Sources

IUPAC 2021 standard atomic weights; CRC Handbook for physical properties.

## Acceptance criteria

- [x] Every entry has a `source`
- [x] `w --why gold.density` shows provenance
- [x] Symbols do not shadow units (`W` the element vs `W` the watt: element needs the entity name)

## 2026-09-23

Done. All 118 elements from PubChem (scripts/data/elements.py): atomic number and mass, density, melting and boiling points where measured (superheavy elements are left without them, not guessed), state, category, year of discovery. Every entry has a source. '--why gold.density' snapshot. Symbols that are units (W, C, K, N, S, V, Pa, Mg, Pb …) are shadowed: '5 W' is watts, '--as element W' is tungsten (kb test and binary test). Etymology exists only for tungsten, with its own citation; the rest can grow.
