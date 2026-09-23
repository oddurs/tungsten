---
id: 69
title: 'Count things in things: how many bananas in a blue whale'
type: feature
status: done
milestone: v0.2.2
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

- `how many bananas in a blue whale`, `how many bananas weigh as much as a blue whale`: a ratio of like quantities
- `mass of 3 apples`: a count after `of`
- `how many calories in a banana`: an article before a thing is not the number 1
- `what's 12% of 80`: `what's` is filler, not a possessive

## Example

```
$ w how many bananas in a blue whale
  ◆ result
  │ 1.269×10⁶
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Every form listed under Behaviour has a snapshot

## 2026-09-23

Done, every form in snapshots. 'how many X in Y' with a thing for X divides Y's measure of X's default property by X: a banana's default is mass, so blue whale mass / banana mass = 1.269×10⁶. 'weigh as much as' reads the same way. A number after 'of' multiplies ('mass of 3 apples' = 546 g). An article before a thing is dropped rather than read as 1 ('how many calories in a banana' = 105 kcal). 'what's'/'who's' at the start is filler. 'how many feet in a mile' still converts, since feet is a unit, not a thing.
