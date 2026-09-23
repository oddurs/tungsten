---
id: 63
title: 'Lean git workflow: one PR per item, local checks, quiet CI'
type: chore
status: done
milestone: v0.2
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
area: infra
effort: s
---

Every change lands as one squash-merged PR per cairn item, on a branch named
after it. Local checks are the gate; CI is one Linux job on main that nobody
waits for.

## Acceptance criteria

- [x] `scripts/check` runs fmt, clippy, tests and `cairn check`
- [x] `scripts/ship` checks, pushes, opens the PR and squash-merges it
- [x] Repo allows squash merges only and deletes merged branches
- [x] CI is a single job on pushes to main
- [x] The workflow is written down where agents and people will read it

## 2026-09-22

Done. scripts/check is the gate; scripts/ship refuses to run on main, with uncommitted changes, or when main has moved, then checks, pushes, opens the PR from the commit and squash-merges it. Repo settings via the API: squash only, PR title and body as the commit, merged branches deleted. The two-OS CI plus separate cairn job became one Linux job on pushes to main that runs scripts/check. This PR is the first shipped with it.
