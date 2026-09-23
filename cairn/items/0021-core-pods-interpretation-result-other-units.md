---
id: 21
title: 'Core pods: interpretation, result, other units'
type: feature
status: done
milestone: v0.1
depends_on:
- 17
- 18
- 19
- 20
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: pods
effort: m
spec: §5
---

## Behaviour

- **interpretation** restates the query with every unit spelled out and every
  implicit operator made explicit.
- **result** is the answer.
- **other units** picks up to 4 alternatives: the best-reading unit from each
  of the quantity's groups (metric, customary, …), then the SI unit. See
  crates/pods/src/other_units.rs.

## Example

```
$ w 60 mph * 2h 15min
  ◆ interpretation
  │ 60 mph × (2 h + 15 min)
  ◆ result
  │ 135 mi
  ◆ other units
  │ 217.3 km  ·  237 600 yd  ·  217 261 m
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Dimensionless results skip the other-units pod

## 2026-09-22

Done. Example updated to the real snapshot. The interpretation keeps canonical symbols (mph) rather than expanding them. 'Other units' shows yd rather than ft because each group offers its best-reading unit (value nearest 1–1000, accepted from ~0.0003 to ~3 million), and yd reads better than ft at 135 mi. Extra groups (nautical, astronomical) speak only when the result is already in them or nothing else fits, which is how c × 1 week shows 1212 au. Dimensionless results and zero skip the pod. Temperatures always get every scale.
