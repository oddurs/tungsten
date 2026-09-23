---
id: 65
title: 'Questions: how tall is X, earth radius, distance from X to Y'
type: feature
status: done
milestone: v0.2.1
assignee: Oddur Sigurdsson
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p0
area: language
effort: m
spec: §2
---

## Behaviour

Answer the questions people type, not only the grammar tungsten already has:

- `how tall/high/heavy/long/wide/deep/far/old/fast/hot is X`, `how much does X weigh`
- `earth radius`, `sun temperature`: an entity followed by a property
- `avogadro's number`, `newton's constant`: names that contain a possessive
- `distance from earth to moon`, `distance between earth and mars`

## Example

```
$ w how tall is the eiffel tower
  ◆ interpretation
  │ height of Eiffel Tower
  ◆ result
  │ 330 m
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Every form listed under Behaviour has a snapshot
- [x] Questions about something that lacks the property explain which properties it has

## 2026-09-23

Done, every form in snapshots: how tall/heavy/far/old/fast/long/hot is X (a leading article is dropped), how much does X weigh, earth radius and big mac calories (entity then property), avogadro's number (phrase matching rejoins 's), distance from earth to moon and distance between earth and moon (either order). 'how tall is gold' gets the no-such-property pod, listing gold's properties. Found and fixed an existing bug: dropping 'the' everywhere (v0.2) had silently broken 'to the power of', since phrases matched after the drop. Now a parser test.
