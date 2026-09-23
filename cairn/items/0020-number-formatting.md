---
id: 20
title: Number formatting
type: feature
status: done
milestone: v0.1
depends_on:
- 9
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: render
effort: m
spec: §6
---

## Behaviour

Thin-space digit grouping, superscript exponents (`1.81×10¹⁴`), 4 significant
figures by default, exact integers never rounded, rationals shown as fractions
when small. Never show more precision than the inputs justify.

## Example

```
$ w speed of light * 1 week in m
  ◆ result
  │ 1.813×10¹⁴ m
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] `--sig N` overrides
- [x] `--plain` uses `e14` instead of superscripts

## 2026-09-22

Done. Snapshot `speed of light * 1 week in m` → 1.813×10¹⁴ m. --sig N overrides (unit tests in render/src/num.rs); --plain gives 1.813e14. Rules: exact integers in full up to 12 digits (6 in other-units), fractions when every input was a whole number, exact decimals up to 12 significant digits, else 4 sig figs keeping integer digits below a million, scientific outside 0.001–999 999. Grouping starts at five digits (SI Brochure §5.4.4), with U+202F, or commas under --plain. Negative numbers use U+2212 when fancy.
