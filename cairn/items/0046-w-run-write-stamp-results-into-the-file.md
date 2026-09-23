---
id: 46
title: 'w run --write: stamp results into the file'
type: feature
status: backlog
milestone: v0.4
depends_on:
- 44
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: notebook
effort: m
spec: §9
---

## Behaviour

Append or replace a `#=> result` comment on each line. Idempotent: running twice
changes nothing the second time.

## Acceptance criteria

- [ ] Idempotence test
- [ ] Preserves the file's line endings and trailing newline
