---
id: 2
key: v0.2
title: Knows things
type: milestone
status: done
depends_on:
- 1
created: 2026-09-22
updated: 2026-09-23
closed_at: 2026-09-23
---

The knowledge base arrives: constants, elements, the solar system and everyday items, addressable by possessive, with the for-scale pod that gives results a sense of size.

## Acceptance criteria

- [x] `w 3 coffees a day for a year in grams of caffeine` matches the concept's headline example
- [x] `w mercury` shows an assuming pod
- [x] `w gold` shows an entity card

## 2026-09-23

v0.2 complete on 2026-09-23, released as 0.2.0. All 12 items done, across PRs #4–#10. Checked against the release binary: the concept's headline query (3 coffees a day for a year in grams of caffeine = 103.8771 g, with coffee at USDA's 94.8 mg), 'mercury' shows an assuming pod, 'gold' shows its card. Knowledge base: 355 CODATA 2022 constants, 118 elements, 31 solar-system bodies, 112 everyday things (43 USDA foods), every value sourced and shown by --why. Honest shortfalls: 112 items rather than ~150, with unverifiable ones left out; etymology exists only for tungsten. 10 more minutes of fuzzing across the new language (1.6M runs) found nothing. CI green on main throughout.
