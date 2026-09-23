---
id: 34
title: Word resolver scoring and the assuming pod
type: feature
status: backlog
milestone: v0.2
depends_on:
- 28
created: 2026-09-22
updated: 2026-09-22
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
$ w mass of mercury
  ◆ assuming
  │ "mercury" is a planet  ·  use --as element for the element
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] `--as <kind>` overrides the resolver
- [ ] Unambiguous queries never show the pod
