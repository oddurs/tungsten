---
id: 73
title: 'Fuel economy: mpg and L/100km'
type: feature
status: backlog
milestone: v0.2.3
created: 2026-09-23
updated: 2026-09-23
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

- [ ] Snapshot test covers the example above, and `5 L/100km in mpg`
- [ ] Converting between unrelated dimensions is still an error
