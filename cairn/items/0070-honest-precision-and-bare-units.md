---
id: 70
title: Honest precision and bare units
type: feature
status: backlog
milestone: v0.2.2
created: 2026-09-23
updated: 2026-09-23
priority: p0
area: render
effort: m
spec: §6
---

## Behaviour

- An answer built from knowledge-base values shows no more significant digits
  than the least precise of them published (at least 4): a blue whale is
  149 685 kg, not 149 685.4821 kg.
- A bare unit (`speed of light`, `mile`) answers in SI: 299 792 458 m/s.
- Food energy is not offered in small calories next to kcal.

## Acceptance criteria

- [ ] Snapshots: mass of a blue whale in kg, speed of light, mile, calories of 3 eggs
- [ ] Unit conversions of exact inputs keep their exact digits (5 mi in km is still 8.04672)
