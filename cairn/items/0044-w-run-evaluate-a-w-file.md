---
id: 44
title: 'w run: evaluate a .w file'
type: feature
status: backlog
milestone: v0.4
depends_on:
- 39
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: notebook
effort: m
spec: §9
---

## Behaviour

Evaluate top to bottom with shared bindings. Print each line with its result
right-aligned in a column. `#` comments pass through; `# heading` lines render
as section rules. An error on one line shows inline and does not stop the run.

## Example

The kitchen-reno example in concept §9.

## Acceptance criteria

- [ ] Snapshot test covers the concept §9 example
- [ ] Exit code reflects whether any line errored
