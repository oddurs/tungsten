---
id: 66
title: 'Substances and cups of things: water, air, ice, flour'
type: data
status: done
milestone: v0.2.1
assignee: Oddur Sigurdsson
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p0
area: kb
effort: m
spec: §4
---

## What to add

Water, ice, seawater, air: density, melting and boiling points where they
apply. Cups of common kitchen staples from USDA (flour, sugar, rice) so
`3 cups of flour` has a mass.

## Sources

NIST Chemistry WebBook for water; U.S. Standard Atmosphere 1976 for air;
USDA FoodData Central for kitchen staples.

## Acceptance criteria

- [x] Every entry has a `source`
- [x] `density of water`, `boiling point of water` and `3 cups of flour in g` answer (snapshots)

## 2026-09-23

Done. Water (density 0.99704702 g/mL at 25 °C, Tanaka 2001; melting 0.00 °C and boiling 99.98 °C at 1 atm, VSMOW), ice (0.9167 g/cm³, CRC), seawater (1.025 kg/L), air (1.225 kg/m³, US Standard Atmosphere 1976), each checked against its source first. A substance's default is its density, so '3 L of water in kg' = 2.991 kg falls out of the existing grammar. USDA cups: flour (125 g), sugar (200 g), uncooked rice (185 g), oats (81 g), plus a teaspoon of salt; the old 'cup of rice' is now explicitly cooked. Snapshots for all criteria.
