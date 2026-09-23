---
id: 17
title: Evaluator with exact rationals
type: feature
status: done
milestone: v0.1
depends_on:
- 13
- 14
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: language
effort: m
spec: §3
---

## Behaviour

Evaluate as exact rationals for as long as possible, falling back to `f64` at
the first irrational operation (`sqrt(2)`, `sin`, non-integer powers). Remember
that exactness was lost so the result pod can say so and the exact-form pod
knows when it has something to show.

## Example

```
$ w 1/3 + 1/6
  ◆ result
  │ 1/2
  ◆ decimal
  │ 0.5
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Type-checking happens before evaluation; mismatches never reach the evaluator
- [x] Division by zero is a pod, not a panic

## 2026-09-22

Done. Snapshot 1/3 + 1/6 gives result 1/2 and a decimal pod with 0.5. check::check_query runs before eval::eval. The checker evaluates only dimensionless exponent subtrees and the operands of a hint, never a mismatched expression. 1/0 renders a can't-divide-by-zero pod with a caret under the 0.
