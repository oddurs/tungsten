---
id: 35
title: For-scale pod
type: feature
status: backlog
milestone: v0.2
depends_on:
- 32
- 33
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: pods
effort: m
spec: §5
---

## Behaviour

For a dimensional result, find the for-scale reference whose ratio is closest to
a round number between ½ and 1000. Show at most two. Prefer references from
different domains (don't show two animals).

## Example

```
$ w 2.5 million L
  ◆ for scale
  │ ≈ 1 Olympic pool
  │ ≈ 8 300 bathtubs
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] Never shows a comparison with a ratio above 1000 or below ½
