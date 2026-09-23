---
id: 36
title: Entity card pod
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: pods
effort: s
spec: §5
---

## Behaviour

A query that is just an entity name renders its properties as an aligned
table.

## Example

```
$ w gold
  ◆ gold · Au · 79
  │ atomic mass    196.96657 u
  │ density        19.282 g/cm³
  │ melting point  1064.18 °C
  │ boiling point  2855.85 °C
  │ state          solid
  │ category       transition metal
  │ discovered     ancient
```

## Acceptance criteria

- [x] Snapshot test covers the example above

## 2026-09-23

Done. A lone name renders a card: an aligned table in properties.toml order, with values in each property's show unit, uncertainties for constants, and facts (state, category, discovered, orbits, named). Snapshots: gold, tungsten, G, the moon, coffee. Example updated to the real snapshot. Exact values keep every significant digit on cards (5.9722×10²⁴ kg, 6.6743×10⁻¹¹), via a number-formatter change covered by tests.
