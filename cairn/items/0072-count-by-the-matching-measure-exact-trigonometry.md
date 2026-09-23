---
id: 72
title: Count by the matching measure; exact trigonometry
type: feature
status: done
milestone: v0.2.3
assignee: Oddur Sigurdsson
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p0
area: language
effort: s
spec: §3
---

## Behaviour

- `how many cups of coffee in a litre`: count by the property whose dimension
  matches what they are counted in (coffee's volume), not the default (caffeine).
- `sin(180°)` is 0, not 1.2×10⁻¹⁶; `tan(90°)` has no value.

## Example

```
$ w how many cups of coffee in a litre
  ◆ result
  │ 4.227
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Snapshots for sin(180°), cos(90°), tan(90°), tan(45°)

## 2026-09-23

Done, all in snapshots. Counting in a quantity uses the thing's property of that dimension: coffee by volume, so a litre holds 4.227 cups. Counting in another thing uses a property both have, X's default first; the interpretation names it ('volume of Lake Superior / volume of Olympic pool'). Trigonometry snaps results within 10⁻¹² of a whole number (sin 180° = 0, cos 90° = 0, tan 45° = 1), and tan at an odd multiple of 90° has no real answer.
