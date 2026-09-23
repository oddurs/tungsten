---
id: 25
title: How loose should English parsing be?
type: decision
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: language
effort: s
spec: §14
---

## Question

Strict parsing is predictable; loose parsing feels magical but fails in
confusing ways. Where on that line does tungsten sit for v0.1?

## Options

1. **Strict + suggestions**: a fixed grammar (§2); anything else fails with a did-you-mean.
2. **Loose**: skip unknown words, guess at intent.
3. **Strict, loosened by corpus**: start strict, log failing queries during dogfooding, promote patterns into the grammar.

## Leaning

Option 3. The interpretation pod makes strictness bearable, because the user
always sees how their words were read.

## Decision

**Option 3: strict, loosened by corpus** (decided 2026-09-22).

- The grammar in concept §2 is the contract. A query outside it fails with a
  caret and a did-you-mean, never a guess.
- The interpretation pod always shows how the words were read, so a strict
  parse is never a silent one.
- Queries that fail during dogfooding go into `tests/queries.txt` as
  expected-error snapshots. A pattern that shows up repeatedly gets promoted
  into the grammar deliberately, with its own snapshot.

## Acceptance criteria

- [x] Decision recorded above and reflected in docs/concept.md
