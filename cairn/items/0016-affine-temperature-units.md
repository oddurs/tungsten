---
id: 16
title: Affine temperature units
type: feature
status: done
milestone: v0.1
depends_on:
- 14
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
area: units
effort: m
spec: §3
---

## Behaviour

`°C`, `°F` and `K` are points on a scale. Differences are `Δ°C`, `Δ°F`. Adding
two absolute temperatures is an error that suggests the delta form.

## Example

```
$ w 20 °C + 5 °C
  ◆ can't add two temperatures
  │ 20 °C + 5 °C
  │ both are readings on a temperature scale; add a difference instead
  │ hint: did you mean 20 °C + 5 Δ°C  (= 25 °C)?
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] `98.6 °F in °C` is `37 °C`
- [x] `°C` in a compound unit (`J/°C`) is treated as a difference

## 2026-09-22

Done. Snapshot matches the example above, which gained the query line and an explanation. 98.6 °F in °C = 37 °C exactly. Semantics: a quantity in a scale unit (K, °C, °F, °R) is a point. Point − point is a difference (30 °C − 20 °C = 10 Δ°C, whose other units show 18 Δ°F). Adding two points errors only when one is on an affine scale, so 300 K + 5 K still works. In a product or quotient a °C/°F reading is a difference: J/°C, and 4.18 J/(g °C) × 100 g × 10 °C = 4.18 kJ (snapshot and core tests). -40 °C is a negative reading, not the negation of 40 °C.
