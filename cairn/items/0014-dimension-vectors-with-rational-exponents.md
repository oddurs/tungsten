---
id: 14
title: Dimension vectors with rational exponents
type: feature
status: done
milestone: v0.1
depends_on:
- 9
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: units
effort: m
spec: §3
---

## Behaviour

A quantity is a value plus a dimension: a vector of rational exponents over
`[length, mass, time, current, temperature, amount, luminosity, currency,
information, count]`. Multiplication adds vectors, powers scale them, addition
requires equality.

`count` is a real dimension so that `3 coffees` is not dimensionless and
`coffees / day` means something.

## Acceptance criteria

- [x] `sqrt(9 m^2)` is `3 m`; `sqrt(2 m)` is `1.414 m^(1/2)`, not an error
- [x] Named derived dimensions (velocity, energy, power, pressure…) for display

## 2026-09-22

Done. Dim is ten rational exponents (crates/units/src/dim.rs). sqrt(9 m^2) = 3 m and sqrt(2 m) = 1.414 m^(1/2), both in snapshots. Named quantities (velocity, energy, power, pressure and 28 more) live in data/units.toml [[quantity]] and drive both error labels and other-units choices.
