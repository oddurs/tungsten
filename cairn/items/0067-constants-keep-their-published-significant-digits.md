---
id: 67
title: Constants keep their published significant digits
type: feature
status: done
milestone: v0.2.1
assignee: Oddur Sigurdsson
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: render
effort: s
spec: §6
---

## Behaviour

CODATA writes G as 6.674 30(15)×10⁻¹¹: the trailing zero is significant. Cards
and results show a knowledge-base value with the digits its source published.

## Example

```
$ w G
  ◆ Newtonian constant of gravitation · G
  │ value        6.674 30×10⁻¹¹ m³/(kg·s²)
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Values computed from data are unaffected

## 2026-09-23

Done. kb::Value.digits counts the significant digits of the value's text. Integers with trailing zeros are ambiguous, so they get None and are formatted as before. On a card, a value shown in its own unit uses NumMode::Published: exactly those digits, trailing zeros kept, fractional digits grouped in threes (SI and CODATA style), published integers in full. Snapshot: G = 6.674 30×10⁻¹¹. Computed values and query results are unaffected; only card lines changed (NASA's 0.33010 → 3.3010×10²³, 3.70, 655.720; CODATA 6.022 140 76). --sig still overrides.
