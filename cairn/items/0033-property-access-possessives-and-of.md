---
id: 33
title: 'Property access: possessives and of'
type: feature
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: language
effort: m
spec: §2
---

## Behaviour

`mass of earth`, `earth's mass` and `earth.mass` are the same AST node. An
item used as a count with a quantity target pulls its `default` property.

## Example

```
$ w density of gold * 1 L
  ◆ result
  │ 19.3 kg
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] `3 coffees in mg of caffeine` works via `default`
