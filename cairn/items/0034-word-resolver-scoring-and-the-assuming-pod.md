---
id: 34
title: Word resolver scoring and the assuming pod
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
area: pods
effort: m
spec: §5
---

## Behaviour

When a word resolves to several things, score candidates by context (does it
type-check? is it next to a unit? a number?) and pick the best. Show the
alternatives in an assuming pod with the flag to pick another.

## Example

```
$ w mercury
  ◆ assuming
  │ "mercury" is a planet  ·  use --as element for the element
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] `--as <kind>` overrides the resolver
- [x] Unambiguous queries never show the pod

## 2026-09-23

Done. The chooser (added in 0033) keeps only candidates that fit the context, then prefers by kind order (star, planet, dwarf planet, moon, element, item, constant). When more than one fit, an assuming pod leads the answer. Example changed from 'mass of mercury' to 'mercury': given the data, 'mass of mercury' is not ambiguous (only the planet has a mass; the element has an atomic mass), and 'melting point of mercury' goes straight to the element. Both are snapshots with no pod. --as <kind> overrides and also unlocks names shadowed by units or constants: --as element W is tungsten, --as constant e is the elementary charge (binary tests). '5 W' shows no pod. Bonus: -q on a card with a default prints its value, so -q G gives 6.6743e-11; tiny exact values use e-notation.
