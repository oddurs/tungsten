---
id: 43
title: 'Meta-commands: :vars :clear :pods :sig :why'
type: feature
status: done
milestone: v0.3
assignee: Oddur Sigurdsson
depends_on:
- 40
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
area: repl
effort: s
spec: §8
---

## Behaviour

The colon commands from concept §8.

## Acceptance criteria

- [x] Snapshot test per command

## 2026-09-23

:vars :clear :pods [on|off] :sig [N] :why :help :quit, each covered by a session snapshot (vars, clear, pods, sig, why, help, quit).
