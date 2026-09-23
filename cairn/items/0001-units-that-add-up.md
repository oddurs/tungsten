---
id: 1
key: v0.1
title: Units that add up
type: milestone
status: done
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
---

The smallest `w` worth using: arithmetic on quantities with units, conversion, and the three core pods, rendered in the filament-glow theme.

## Acceptance criteria

- [x] `w 60 mph * 2h 15min in km` prints interpretation, result and other-units pods
- [x] `w 3 m + 2 s` prints the dimension-mismatch diagram
- [x] `w -q 5 mi in km` prints `8.04672` and nothing else
- [x] Every item filed here is done or moved

## 2026-09-22

v0.1 complete on 2026-09-22. All 19 items are done; one criterion moved to 0039, where it belongs. Checked against the release binary: interpretation/result/other-units pods; the mismatch diagram; `-q 5 mi in km` prints exactly 8.04672. 72 tests (unit, 48-query snapshot corpus at two widths, binary integration, README sync); clippy -D warnings and fmt clean; 12+ minutes of fuzzing without a crash. Not tagged: the repository has no commits yet.
