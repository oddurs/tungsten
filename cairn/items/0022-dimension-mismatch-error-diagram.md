---
id: 22
title: Dimension-mismatch error diagram
type: feature
status: done
milestone: v0.1
depends_on:
- 17
- 19
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: render
effort: m
spec: §7
---

## Behaviour

Type errors render as a pod with brackets under each operand naming its
dimension, plus a hint when one operator change would make the query valid.

## Example

```
$ w 3 m + 2 s
  ◆ can't add these
  │ 3 m + 2 s
  │ ─┬─   ─┬─
  │  │     └ time
  │  └ length
  │ hint: did you mean 3 m / 2 s  (= 1.5 m/s)?
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Exit code 2 for query errors, distinct from 1 for internal errors

## 2026-09-22

Done. The snapshot for 3 m + 2 s is character-for-character the spec's diagram (pods errors::tests::mismatch_diagram_matches_the_spec). Exit code 2 for any query error; a panic hook prints 'internal error' and exits 1. The hint is offered only when the quotient or product is a named quantity.
