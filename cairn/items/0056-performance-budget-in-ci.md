---
id: 56
title: Performance budget in CI
type: chore
status: backlog
milestone: v1.0
depends_on:
- 11
created: 2026-09-22
updated: 2026-09-22
priority: p0
area: infra
effort: m
spec: §12
---

Cold start under 5 ms and any corpus query under 1 ms, measured with
`hyperfine` in CI on Linux.

## Acceptance criteria

- [ ] CI fails on regression beyond 20%
