---
id: 35
title: For-scale pod
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 32
- 33
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: pods
effort: m
spec: §5
---

## Behaviour

For a dimensional result, find the for-scale reference whose ratio is closest to
a round number between ½ and 1000. Show at most two. Prefer references from
different domains (don't show two animals).

## Example

```
$ w 2.5 million L
  ◆ for scale
  │ ≈ 1 Olympic pool
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Never shows a comparison with a ratio above 1000 or below ½

## 2026-09-23

Done. crates/pods/src/scale.rs compares a dimensional result with every reference tagged 'scale' (34 items and foods, plus the Sun, Earth, Moon and Jupiter, now tagged in scripts/data/solar.py). Only ratios from ½ to 1000 count, enforced and tested across lengths, masses, areas, volumes, data and time. Ratios near a round number (1, 1.5, 2, 2.5, 3, 4, 5, 7.5 × 10ⁿ) win, with a slight lean to small ones. At most two, never from the same domain, never the thing asked about (test: volume of olympic pool). The property is named when a reference lends several: '≈ 2 Moons (radius)'. Example changed: the bathtub was dropped in 0032 for lack of a checkable source, and nothing else sits near 2.5 million L, so the snapshot shows one line. Other snapshots: 100 t ≈ 2.8 semi trucks and 0.7 × blue whale; mass of earth ≈ 81 Moons (mass); mass of jupiter ≈ 320 Earths; 70 yr ≈ 1 human lifetime. --plain renders ≈ as ~ (the binary test caught it).
