---
id: 74
key: v0.3.1
title: Polish IV
type: milestone
status: done
assignee: Oddur Sigurdsson
depends_on:
- 3
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
---

Dogfooding the v0.3 session features: statements that fail stop the line
instead of cascading, refusals suggest a name that works, definitions explain
their mistakes, and the REPL can change what an ambiguous name means.

## Acceptance criteria

- [x] `t = 20 °C; t in °F` reports one error, about `t`, and suggests a name
- [x] `f(2) = 3` and `f(x, x) = x` explain what a definition needs
- [x] `:as element` in the REPL makes `mercury` the element, and the assuming hint says so
- [x] `n = 3; n dozen` is 36

## 2026-09-23

Released as 0.3.1. A failing statement now stops its line (CLI, stdin and REPL), so 't = 20 °C; t in °F' reports one error instead of a cascade. Refused names suggest a free one (m → m1, t → t1; none for keywords). Definitions explain malformed heads: no parameters, non-name parameters, duplicates, empty bodies; 'x =' asks for a value. Magnitude words stand alone ('n dozen', 'dozen' = 12). ':as KIND' in the REPL, and the assuming hint names ':as' there. Checked: completion under 1 ms per Tab in release; session fuzz 2 min clean.
