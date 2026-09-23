---
id: 24
title: 'CLI: arguments, -q, --plain, stdin'
type: feature
status: done
milestone: v0.1
depends_on:
- 9
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: cli
effort: s
spec: §6
---

## Behaviour

`w <query words…>` joins arguments with spaces so quoting is rarely needed.
`-q` prints only the result value. `--plain` gives ASCII and no colour. With no
arguments and a non-TTY stdin, evaluate one query per line.

## Example

```
$ echo "3 ft in cm" | w -q
91.44
```

## Acceptance criteria

- [x] Snapshot test covers the example above
- [x] `w --help` fits in 24 lines
- [x] Shell-hostile characters (`*`, `?`, `'`) are documented in `--help`

## 2026-09-22

Done. crates/cli/tests/cli.rs runs the real binary. `echo "3 ft in cm" | tungsten -q` → 91.44 (inline snapshot). Also covered: exit code 2 on a query error, with the one-line reason on stderr; stdin skips blank lines and # comments; one bad line fails the run without stopping the others; -40 is a number, not a flag; no colour when piped; --plain is pure ASCII. --help is 21 lines, and its closing paragraph documents * ? ' " ( ) and the words to use instead.
