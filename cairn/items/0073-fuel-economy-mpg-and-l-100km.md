---
id: 73
title: 'Fuel economy: mpg and L/100km'
type: feature
status: done
milestone: v0.2.3
assignee: Oddur Sigurdsson
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: units
effort: m
spec: §2
---

## Behaviour

Miles per gallon (distance per volume) and litres per 100 km (volume per
distance) are reciprocal dimensions, so converting between them inverts the
value. `L/100km` is written as people write it.

## Example

```
$ w 20 mpg in L/100km
  ◆ result
  │ 11.76 L/100km
```

## Acceptance criteria

- [x] Snapshot test covers the example above, and `5 L/100km in mpg`
- [x] Converting between unrelated dimensions is still an error

## 2026-09-23

Done. mpg (mi/US gal) and L/100km (0.01 L/km), with a lexer rule so 'L/100km' and 'l/100 km' are one unit as written. When the only conversion target has the reciprocal dimension, the value is inverted: 20 mpg = 11.76 L/100km, 5 L/100km = 47.04 mpg, 8 l/100 km = 29.4 mpg (snapshots). Unrelated dimensions still error ('1 m in s'). Fuel consumption has the dimension of area and an inverted answer the reciprocal of its value's, so neither gets other units or for-scale lines. A 'fuel economy' quantity offers km/L for mpg.
