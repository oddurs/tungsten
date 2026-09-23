---
id: 54
title: Dates and durations
type: feature
status: backlog
milestone: v0.6
depends_on:
- 17
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: language
effort: m
spec: §10
---

## Behaviour

ISO dates, `today`, `days until <date>`, `<date> ± <duration>`, durations in
calendar units. No time zones.

## Example

```
$ w 2026-09-22 + 90 days
  ◆ result
  │ Mon 21 Dec 2026
```

## Acceptance criteria

- [ ] Snapshot test covers the example above (with a pinned clock)
- [ ] `1 month` is ambiguous and says so in an assuming pod
