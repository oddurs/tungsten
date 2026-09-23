---
id: 11
title: 'CI: build, test, clippy, cairn check'
type: chore
status: done
milestone: v0.1
depends_on:
- 10
created: 2026-09-22
updated: 2026-09-22
closed_at: 2026-09-22
priority: p1
area: infra
effort: s
---

GitHub Actions on macOS and Linux.

## Acceptance criteria

- [x] `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`
- [x] `cairn check --strict --render` so ROADMAP.md cannot drift

## 2026-09-22

Workflow written: .github/workflows/ci.yml. Job 'test' runs on ubuntu and macos: fmt --check, clippy -D warnings, cargo test (every snapshot plus the README check, with INSTA_UPDATE=no so drift fails). Job 'cairn' installs cairn with its install.sh and runs check --strict --render. The repo has no GitHub remote yet, so the workflow has not run on Actions. Every step was run locally and passes (72 tests, clippy clean, cairn check ok), and the YAML parses. Watch the first real run when a remote exists.
