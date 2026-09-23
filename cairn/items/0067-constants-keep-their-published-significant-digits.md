---
id: 67
title: Constants keep their published significant digits
type: feature
status: backlog
milestone: v0.2.1
created: 2026-09-23
updated: 2026-09-23
priority: p1
area: render
effort: s
spec: §6
---

## Behaviour

CODATA writes G as 6.674 30(15)×10⁻¹¹: the trailing zero is significant. Cards
and results show a knowledge-base value with the digits its source published.

## Example

```
$ w G
  ◆ Newtonian constant of gravitation · G
  │ value        6.674 30×10⁻¹¹ m³/(kg·s²)
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] Values computed from data are unaffected
