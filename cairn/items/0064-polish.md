---
id: 64
key: v0.2.1
title: Polish
type: milestone
status: done
depends_on:
- 2
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
---

Rough edges found by dogfooding v0.2 with the questions people actually ask.

## Acceptance criteria

- [x] `how tall is the eiffel tower` and `how much does a blue whale weigh` answer
- [x] `density of water` answers
- [x] G shows its published digits, 6.674 30×10⁻¹¹

## 2026-09-23

v0.2.1 complete on 2026-09-23, released as 0.2.1 (PRs #12–#14 plus this one). Checked against the release binary: 'how tall is the eiffel tower' = 330 m, 'how much does a blue whale weigh' = 330 000 lb, 'density of water' = 0.99704702 g/mL, G = 6.674 30×10⁻¹¹. Also fixed along the way: 'to the power of', broken since v0.2. Fuzzed 5 minutes (632,547 runs) with no crashes.
