---
id: 53
title: Uncertainty propagation and pod
type: feature
status: backlog
milestone: v0.6
depends_on:
- 17
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: units
effort: l
spec: §3 §5
---

## Behaviour

`±` values propagate by first-order (linear) error propagation, assuming
independence. The uncertainty pod shows relative error and which input
dominates the variance. CODATA constants carry their stored uncertainty.

## Example

```
$ w (5.0 ± 0.2) m / (1.3 ± 0.1) s
  ◆ result
  │ 3.8 ± 0.3 m/s
  ◆ uncertainty
  │ relative 8.7%  ·  dominated by time (79% of variance)
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] Results rounded to the uncertainty's leading digit
