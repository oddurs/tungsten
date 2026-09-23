---
id: 19
title: Pod layout and filament-glow theme
type: feature
status: done
milestone: v0.1
depends_on:
- 9
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: render
effort: m
spec: §1
---

## Behaviour

The pod renderer: `◆` title, `│` gutter, footer rule with `W74 · <time>`. Amber
accent, cyan units, dim interpretation. Width-aware, with a single-column
layout below 60 columns. `NO_COLOR` and non-TTY output drop all styling.

## Acceptance criteria

- [x] Snapshot test at widths 80 and 40
- [x] Theme is one struct; no ANSI codes outside the render crate
- [x] Looks right on dark and light terminal backgrounds (manual check, screenshots in PR)

## 2026-09-22

Done. Theme is one struct (crates/render/src/theme.rs, FILAMENT); no ANSI escapes exist outside the render crate (grepped). Snapshots at 80 and 40. Checked on both backgrounds with docs/screenshots/{dark,light}.png, rendered from real ANSI output by scripts/screenshots.py (there is no PR to attach them to). Amber moved from #d78700 to #af8700 because the former was 2.9:1 on white; every colour now clears 3:1 on black and white. Result values are bold in the terminal's own foreground, not white, so they survive light themes.
