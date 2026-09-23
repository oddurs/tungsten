# tungsten

A Wolfram|Alpha-flavoured calculator for the terminal, focused on quantities
and units. The design lives in `docs/concept.md`; the roadmap lives in cairn
(`cairn/items/`, rendered to `ROADMAP.md`).

- Every item's `spec` field names the concept section it implements. Read that
  section before starting the item.
- A feature's `## Example` block is the snapshot test that proves it done.
- If the implementation departs from `docs/concept.md`, update the concept in
  the same change.

## Git workflow

One cairn item, one branch, one squash-merged PR. `main` is linear and always
passes `scripts/check`.

```sh
cairn claim 0028
git switch -c 0028-kb-build-pipeline     # <item id>-<short name>
# ...work, commit...                     # close the item in the same branch
scripts/ship                             # check, push, PR, squash-merge, back on main
```

- `scripts/check` is the gate: fmt, clippy, tests (including every snapshot
  and the README examples), `cairn check`. Run it whenever; `ship` runs it too.
- Commit subjects are imperative and name the change, not the item number
  (`Compile the knowledge base into phf tables`). With one commit per branch,
  its subject and body become the PR and the squash commit.
- The item's status, notes and ticked criteria change in the same PR as the
  code, so history shows both together.
- CI is one Linux job on `main`, a safety net after merge. Nothing waits for it.
- Never commit to `main` directly, never force-push `main`.

<!-- cairn:begin -->
## Roadmap and issues

This project tracks its roadmap and issues with `cairn`. Every item is a Markdown file under `cairn/items`, described by the schema in `cairn.toml`.

**Do not create ad-hoc TODO, PLAN or NOTES files.** Create a cairn item instead, so the work appears on the board and in the generated roadmap.

### The loop

1. `cairn next` — what is ready to start. It excludes anything blocked by unfinished dependencies and puts work already in progress first.
2. `cairn claim <ID>` — take it before you start, so no one duplicates the work. `cairn claim --next` picks and claims the top-ranked unclaimed item in one step, and prints its body so you can begin immediately.
3. Do the work. Record what you learn: `cairn set <ID> <field>=<value>` for fields, `cairn note <ID> "<TEXT>"` for anything that needs a sentence — why you chose something, what you tried, what to watch for.
4. `cairn tick <ID> <N>` as each acceptance criterion becomes true — `cairn show <ID> --criteria` lists them numbered. Tick what is true, not what would let you close.
5. `cairn close <ID>` when it is done, or `cairn release <ID>` to hand it back.
6. `cairn check` before you report finished. It must pass.

### Commands

```sh
cairn next --json                 # ready work, ranked
cairn claim --next                # take the next ready item
cairn search <TEXT> --json        # titles, bodies and labels
cairn list --json                 # all open items
cairn list --filter 'blocked=false,priority=p0'
cairn show <ID> --json            # one item, including its body
cairn new "<TITLE>" --type <TYPE> --milestone <MILESTONE>
cairn set <ID> status=<STATUS>    # also labels+=x, or any field below
cairn note <ID> "<TEXT>"          # append reasoning; never replaces
cairn show <ID> --criteria        # acceptance criteria, numbered
cairn tick <ID> <N>               # tick one; --all for every one
cairn close <ID>
cairn check                       # validate; run before finishing
cairn render                      # regenerate ROADMAP.md
```

### Schema

- **Types**: `feature`, `bug`, `data`, `decision`, `chore`, `docs`, `milestone`
- **Statuses**: `backlog` (open), `planned` (open), `doing` (active), `blocked` (active), `done` (done), `dropped` (dropped)
- **`area`**: one of language, units, kb, pods, render, cli, repl, notebook, infra, docs — Subsystem, matching the crate layout (concept §12)
- **`priority`**: one of p0, p1, p2, p3 — p0 blocks its milestone's release
- **`effort`**: one of s, m, l, xl — s: an evening · m: a weekend · l: a week · xl: split it
- **`spec`**: free text — Section of docs/concept.md this implements, e.g. §5
- **`due`**: date, YYYY-MM-DD — When a milestone is meant to land
- **Milestones**: `v0.1`, `v0.2`, `v0.3`, `v0.4`, `v0.5`, `v0.6`, `v1.0`, `later`
- **Saved views** (`cairn list --view NAME`): `now`, `next`, `decisions`, `data`, `bugs`, `triage`

### Rules

1. Before starting work, find or create the item and set it to an active status.
2. Use the fields above rather than inventing new ones; add new fields to `cairn.toml` first.
3. Never hand-edit the generated roadmap file — change items and run `cairn render`.
4. `cairn check` must pass before the work is considered done.

<!-- cairn:end -->
