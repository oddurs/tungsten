---
id: 13
title: Pratt parser with implicit multiplication
type: feature
status: done
milestone: v0.1
depends_on:
- 12
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: language
effort: m
spec: §2
---

## Behaviour

Precedence, loosest first: conversion (`in` `to` `as`) → `+ -` → `* /` →
unary `-` → implicit multiplication → `^` → postfix (`%` `!` `squared`).

Implicit multiplication binds tighter than `/`, so `3 m / 2 s` is
`(3 m)/(2 s)` and `60 km/h` is `(60 km)/h`. A fraction of two bare number
literals binds tightest of all, so `1/2 km` is half a kilometre. English
operators from §2 (`per`, `a`, `of`, `times`) desugar in the parser, not the
lexer, so `a` can still be a variable.

## Example

```
$ w 1/2 km in m
  ◆ result
  │ 500 m
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] `3 a day` parses as `3 / day` (variables are v0.3; `a = 2; a * 3` moved to 0039)
- [x] `20% of 80` is `16`
- [x] Parser never panics (fuzz for 10 minutes with cargo-fuzz)

## 2026-09-22

Done. Snapshot: 1/2 km in m → 500 m. 3 a day → 3 per day; 20% of 80 → 16 (both snapshots). Fuzzed with cargo-fuzz (fuzz/, target 'query'), which drives the whole pipeline through rendering, not just the parser: 2,168,569 runs in 601 s with no crashes, then 454,520 runs in 121 s after the word-wrap rewrite. Behaviour text corrected: the old '1/2 km' explanation had the precedence backwards; the real rule is that fractions of bare numbers bind tightest. Also handled: mixed numbers (3 1/2), 2^1/2 = 1, compound quantities, 'how many X in Y', and '100 in cm' as inches. The `a = 2` criterion moved to 0039 because variables are v0.3.
