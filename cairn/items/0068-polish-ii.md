---
id: 68
key: v0.2.2
title: Polish II
type: milestone
status: done
depends_on:
- 64
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
---

The second dogfooding pass: counting things in other things, and answers that claim no more precision than their sources.

## Acceptance criteria

- [x] `how many bananas in a blue whale` answers
- [x] `mass of a blue whale in kg` shows no more precision than NOAA gave
- [x] `speed of light` answers in m/s

## 2026-09-23

v0.2.2 complete on 2026-09-23, released as 0.2.2 (PRs #16–#17 plus this one). Checked against the release binary: 'how many bananas in a blue whale' = 1.269×10⁶, 'mass of a blue whale in kg' = 149 685 kg, 'speed of light' = 299 792 458 m/s. Fuzzed 4 minutes (388,794 runs) with no crashes.
