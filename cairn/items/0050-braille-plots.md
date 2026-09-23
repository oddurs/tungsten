---
id: 50
title: Braille plots
type: feature
status: backlog
milestone: v0.5
depends_on:
- 17
- 19
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: render
effort: l
spec: §5
---

## Behaviour

`plot f(x) from a to b` draws on a braille canvas (2×4 dots per cell) with axis
labels. Handles discontinuities without drawing vertical spikes.

## Example

```
$ w plot sin(x)/x from -10 to 10
```

## Acceptance criteria

- [ ] Snapshot test covers the example above
- [ ] `tan(x)` does not draw asymptote spikes
- [ ] `--plain` falls back to ASCII `*` plotting
