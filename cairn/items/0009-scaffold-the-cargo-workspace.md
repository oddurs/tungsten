---
id: 9
title: Scaffold the Cargo workspace
type: chore
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p0
area: infra
effort: s
spec: §12
---

Create the workspace with the six crates from concept §12 (`core`, `units`, `kb`,
`pods`, `render`, `cli`), a `data/` directory, and the binary named `tungsten`.

## Acceptance criteria

- [x] `cargo build` produces a `tungsten` binary that prints a placeholder pod
- [x] Crate dependency direction is one-way: cli → pods/render → core → units
- [x] `rustfmt.toml` and `clippy` lints configured

## 2026-09-22

Done. Six crates under crates/ (units, kb, core, pods, render, cli), binary `tungsten`. Runtime dependencies run one way: cli → render → pods → core → units (kb → units; render's dev-dependency on core is tests only). rustfmt.toml, clippy.toml and [workspace.lints] configured; `cargo clippy -D warnings` is clean. The binary went straight past a placeholder pod to real ones.
