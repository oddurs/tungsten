---
id: 36
title: Entity card pod
type: feature
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: pods
effort: s
spec: §5
---

## Behaviour

A query that is just an entity name renders its properties as an aligned
table.

## Example

```
$ w gold
  ◆ gold · Au · 79
  │ atomic mass   196.97 u
  │ density       19.3 g/cm³
  │ melting point 1064 °C
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
