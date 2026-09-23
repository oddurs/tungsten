---
id: 23
title: Did-you-mean for unknown words
type: feature
status: done
milestone: v0.1
depends_on:
- 12
- 15
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
area: language
effort: s
spec: §7
---

## Behaviour

Unknown words get the closest unit, entity or variable by edit distance
(Damerau–Levenshtein ≤ 2), and the caret points at the word.

## Example

```
$ w 5 kilometers in milez
  ◆ unknown word
  │ 5 kilometers in milez
  │                 ^^^^^ did you mean miles?
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] No suggestion when nothing is within distance 2

## 2026-09-22

Done. Snapshot matches the example exactly. Optimal-string-alignment distance ≤ 2 (≤ 1 for words of four letters or fewer) over every unit symbol, unit name and keyword. Case-insensitive, so KM → km. zzzzzz gets no suggestion (test).
