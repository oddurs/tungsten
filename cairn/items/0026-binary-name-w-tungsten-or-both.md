---
id: 26
title: 'Binary name: w, tungsten, or both'
type: decision
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: cli
effort: s
spec: §14
---

## Question

`w` is the ideal thing to type but shadows the Unix `w` (who is logged in).

## Options

1. Ship `tungsten`, document `alias w=tungsten`.
2. Ship `w` and accept the shadowing.
3. Ship `tungsten` plus an opt-in `tungsten --install-alias`.

## Leaning

Option 1. Clobbering a POSIX command on install is rude; an alias is the
user's choice to make.

## Decision

**Option 1: the binary is `tungsten`** (decided 2026-09-22).

Documentation and examples write `w`, and the README's first instruction is
`alias w=tungsten` (fish: `abbr -a w tungsten`). Installing never shadows
POSIX `w`; the short name is the user's choice.

## Acceptance criteria

- [x] Decision recorded above and reflected in docs/concept.md
