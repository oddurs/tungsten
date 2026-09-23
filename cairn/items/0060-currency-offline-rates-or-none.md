---
id: 60
title: 'Currency: offline rates or none?'
type: decision
status: backlog
milestone: later
created: 2026-09-22
updated: 2026-09-22
priority: p2
area: cli
effort: s
spec: §14
---

## Question

Currency conversion is a top Wolfram|Alpha use, but rates go stale, which
breaks "correct", and fetching them breaks "offline".

## Options

1. No currency conversion at all; `usd` works only as a unit of account.
2. Opt-in `w update-rates` caches rates locally; results always show the rates' date.
3. Bundle rates at build time.

## Leaning

Option 2, with the date shown in the result pod, not just a footnote.

## Decision

## Acceptance criteria

- [ ] Decision recorded above and reflected in docs/concept.md
