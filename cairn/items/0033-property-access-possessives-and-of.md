---
id: 33
title: 'Property access: possessives and of'
type: feature
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
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
  ◆ interpretation
  │ density of gold × 1 L
  ◆ result
  │ 19 282 g
  ◆ other units
  │ 19.28 kg  ·  42.51 lb
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] `3 coffees in mg of caffeine` works via `default`

## 2026-09-23

Done. Snapshots cover the example (updated to the real output: density of gold × 1 L = 19 282 g; L·cm⁻³ now cancels during unit simplification) and '3 coffees in mg of caffeine' = 285 mg via coffee's default. Also: earth's mass, earth.mass, 'the mass of the earth' ('the' is dropped everywhere), 'how much caffeine in 3 coffees', and the concept's headline query (104.09625 g, exact with a Julian year). A new chooser pass (crates/core/src/entities.rs) picks which entity a name means from what the context needs. Values given in a unit stay exact when shown in that unit: Value.given, needed because u → kg overflows i128.
