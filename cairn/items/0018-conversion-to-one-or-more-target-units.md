---
id: 18
title: Conversion to one or more target units
type: feature
status: done
milestone: v0.1
depends_on:
- 15
- 17
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: units
effort: m
spec: §2
---

## Behaviour

`in` / `to` / `as` converts the whole expression. A comma list gives a
mixed-unit answer, largest first.

## Example

```
$ w 100000 s in h, min, s
  ◆ result
  │ 27 h 46 min 40 s
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Converting to an incompatible unit shows both dimensions
- [x] `5 mi in km` is `8.04672 km` exactly (rational path)

## 2026-09-22

Done. 100000 s in h, min, s gives 27 h 46 min 40 s (snapshot). 5 km in s shows both dimensions: 'the query is a length, but s measures time'. 5 mi in km = 25146/3125 km exactly (core test headline_examples); -q prints 8.04672.
