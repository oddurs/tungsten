---
id: 70
title: Honest precision and bare units
type: feature
status: done
milestone: v0.2.2
assignee: Oddur Sigurdsson
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] Snapshots: mass of a blue whale in kg, speed of light, mile, calories of 3 eggs
- [x] Unit conversions of exact inputs keep their exact digits (5 mi in km is still 8.04672)

## 2026-09-23

Done. precision() finds the fewest significant digits among the knowledge-base values an answer used (an integer's trailing zeros don't count, so NOAA's 330,000 lb has 2). The result pod then shows at most max(that, 4) digits; exact integers stay whole and --sig still wins. Snapshots: blue whale 149 685 kg (was 149 685.4821), the headline coffee query 103.9 g (coffee's 94.8 mg has 3 digits), 3 L of water 2.9911411 kg (water's density has 8). Answers with no data are unchanged: 5 mi in km is still 8.04672. A bare unit answers in SI (speed of light = 299 792 458 m/s, mile = 1609.344 m, hour = 3600 s); temperature scales are left alone. Small calories are no longer offered next to kcal.
