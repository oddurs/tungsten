---
id: 48
title: 'solve: polynomials up to degree 4, with steps'
type: feature
status: backlog
milestone: v0.5
depends_on:
- 17
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: language
effort: l
spec: §5
---

## Behaviour

Solve a single-variable polynomial equation. Exact roots where they exist
(factoring, quadratic formula), numeric otherwise. Steps pod shows the
rearrangement and factorisation.

## Example

```
$ w solve x^2 - 3x = 10
  ◆ solutions
  │ x = 5    x = −2
  ◆ steps
  │ x² − 3x − 10 = 0
  │ (x − 5)(x + 2) = 0
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] Complex roots shown as `a ± bi`
- [ ] Units on both sides are checked (`solve 2x = 10 m` gives `x = 5 m`)
