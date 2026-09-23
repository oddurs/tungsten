---
id: 71
key: v0.2.3
title: Polish III
type: milestone
status: done
depends_on:
- 68
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
---

Third dogfooding pass: counting by the right measure, trigonometry at exact angles, fuel economy.

## Acceptance criteria

- [x] `how many cups of coffee in a litre` compares volumes
- [x] `tan(90°)` is undefined and `sin(180°)` is 0
- [x] `20 mpg in L/100km` answers

## 2026-09-23

v0.2.3 complete on 2026-09-23, released as 0.2.3 (PRs #19–#20 plus this one). Checked against the release binary: 'how many cups of coffee in a litre' = 4.227, 'tan(90°)' has no real answer, '20 mpg in L/100km' = 11.76 L/100km. Fuzzed 4 minutes (541,335 runs) with no crashes.
