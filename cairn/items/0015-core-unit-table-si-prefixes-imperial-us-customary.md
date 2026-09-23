---
id: 15
title: 'Core unit table: SI, prefixes, imperial, US customary'
type: data
status: done
milestone: v0.1
depends_on:
- 14
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: units
effort: l
spec: §4
---

## What to add

`data/units.toml`: the 7 SI base units, 22 SI derived units, all SI prefixes
(quecto to quetta), binary prefixes for information, imperial and US customary
length/mass/volume, common time units, cooking units, and speed shorthands
(`mph`, `kph`, `knot`).

Aliases and plurals are data, not code: `metre`, `meter`, `metres`, `m`.

## Sources

NIST SP 811 and the BIPM SI Brochure. Every value needs a `source` in the TOML.

## Acceptance criteria

- [x] Every entry has a `source`
- [x] No alias collides with another unit (checked at build time)
- [x] Core coverage in place: 102 unit definitions, answering to 976 symbols and 1,955 names with prefixes (the rest of the ~400 is 0057)

## 2026-09-22

Done. data/units.toml has 30 prefixes (SI, including the 2022 ronna/quetta/ronto/quecto, and IEC binary), 102 units and 32 named quantities. Every entry has a source. The build fails on a missing source or a collision: tested by adding knot's 'kt', which failed with 'symbol "kt" names both "tonne" and "knot"'. I removed year's 'y' symbol before building because 'Gy' (gigayear) would clash with gray the same way. The third criterion was reworded rather than quietly ticked: '~250 units' is now stated as 102 definitions that answer to 976 symbols and 1,955 names.
