---
id: 30
title: The 118 elements
type: data
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
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

- [ ] Every entry has a `source`
- [ ] `w --why gold.density` shows provenance
- [ ] Symbols do not shadow units (`W` the element vs `W` the watt: element needs the entity name)
