---
id: 12
title: Normaliser and lexer
type: feature
status: done
milestone: v0.1
depends_on:
- 9
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: language
effort: m
spec: §2 §3
---

## Behaviour

Normalise before tokenising: unicode operators (`×` `÷` `−` `²` `³` `½`), smart
quotes, digit grouping (`1,500`), magnitude suffixes (`1.5k`, `2.4 million`),
percent, and filler words (`what is`, `how many`, trailing `?`).

Tokens carry byte spans back into the *original* input, because error carets
and the interpretation pod both point at what the user typed.

## Example

```
$ w what is 1,500 × 2²?
  ◆ result
  │ 6000
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] Spans survive normalisation (property test: every token span is a valid slice of the input)
- [x] `5'11"` lexes as feet and inches, not a string

## 2026-09-22

Done. Example snapshot shows 6000, not 6 000: grouping starts at five digits, per the SI Brochure. Spans are byte ranges into the original input, and a proptest (lex::tests::spans_are_valid_slices, plus one over math-like input) checks every span is a valid slice. 5'11" lexes as Num Foot Num Inch. Also: plain spaces group digits (1 000 000) when the groups are valid, which fixed a query silently reading as 1 × 0 × 0.
