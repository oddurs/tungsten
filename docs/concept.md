# tungsten — concept

Tungsten is element 74, symbol **W**, from its old name *wolfram*. The tool is a
terminal nod to Wolfram|Alpha, deliberately narrowed to the part people use it
for most: **calculating with real-world quantities**, typed as a mix of English
and math, answered in a stack of *pods*.

```
$ w 3 coffees a day for a year in grams of caffeine

  ◆ interpretation
  │ 3 × coffee(94.8 mg caffeine) / d × 1 yr  →  g of caffeine

  ◆ result
  │ 103.8771 g

  ◆ other units
  │ 0.1039 kg  ·  3.664 oz

  ◆ for scale
  │ ≈ 2.1 eggs
  │ ≈ 42 US pennies

  ─────────────────────────────── W74 · 0.4ms
```

The pitch is **offline, instant, correct**. No live data, no network, no
accounts. A small curated knowledge base compiled into one binary.

## §1 Visual identity

"Filament glow": one warm amber accent on neutral greys.

| Element | Style |
|---|---|
| Pod glyph `◆` and titles | amber `#af8700`, bold |
| Answer values | bold, in the terminal's own foreground colour |
| Units | teal `#008787` |
| Hints, labels, separators, footer | grey `#808080` |
| Errors | muted red `#d75f5f`, with a caret `^` under the problem |
| Footer | `W74 · 0.4ms`, grey |

Every colour keeps at least 3:1 contrast on both black and white backgrounds,
because a terminal's background is unknowable. Values use bold rather than
white for the same reason. `scripts/screenshots.py` renders
`docs/screenshots/{dark,light}.png` to check.

- Adapts to terminal width; single-column pods below 60 columns.
- Respects `NO_COLOR`; no styling when stdout is not a terminal.
- No spinners, no emoji. Polish comes from alignment and spacing.

## §2 The language

Anything you would type into a search box should mostly work. Anything you would
write on a whiteboard should definitely work.

**Numbers:** `1500` `1,500` `1 500` `1.5k` `1.5e3` `3/4` `3 1/2` `50%` `2.4 million` `½` `2½` `5.0 ± 0.2`

**Quantities:** `5 km` `2h 15min` `5'11"` `3 cups` `60 mph` `9.81 m/s^2`

**English that maps onto operators:**

| You type | Meaning |
|---|---|
| `per`, `a`, `each` | division: `3 a day` → `3 / day` |
| `of` | multiplication: `20% of 80`, `half of 3 km` |
| `times`, `x` | multiplication |
| `squared`, `cubed` | `^2`, `^3` |
| `in`, `to`, `as` | conversion — lowest precedence, always applies last |
| `what is`, `how many`, `?` | ignored |

**Properties:** `mass of earth` · `earth's mass` · `earth.mass` · `earth mass`

**Questions:** `how tall is the eiffel tower` · `how much does a blue whale weigh`
· `how far is the moon` · `distance from earth to moon` · `how old is the universe`

**Variables and functions:** `rent = 2400 usd/month` · `f(x) = x^2 - 3x` · `it`

**Commands** (first word switches mode): `solve` `plot` `table` `compare`
`estimate` `exact` `steps`

```
line       := command? expr (conversion)?
conversion := ("in"|"to"|"as") unit ("," unit)*
expr       := Pratt, loosest first:  + -  <  * / of per  <  unary -  <  implicit-mult  <  ^  <  postfix(%, !, squared)
atom       := number | quantity | name | name "(" args ")" | possessive | "(" expr ")"
```

Implicit multiplication binds tighter than `/`, so `3 m / 2 s` is `(3 m)/(2 s)`
and `60 km/h` is `(60 km)/h`. A fraction of two bare number literals binds
tightest of all, so `1/2 km` is half a kilometre rather than `1/(2 km)`.
Several quantities in a row with the same dimension add: `2h 15min`, `5 ft 11 in`.

## §3 Evaluation pipeline

```
raw text
  → normalise      unicode ², ×, ÷, smart quotes, "1,000", strip filler words
  → tokenise       numbers, words, operators
  → resolve words  unit | entity | variable | command | function, scored by context;
                   ambiguous words become "assuming" candidates
  → parse          Pratt parser → AST
  → type-check     every node carries a dimension vector; mismatches fail here
  → evaluate       exact rationals while possible, then f64
  → choose pods    by kind of result
  → render
```

- **Dimensions** are rational exponent vectors over
  `[length, mass, time, current, temperature, amount, luminosity, currency, information, count]`.
  `count` is what lets "3 coffees" work.
- **Temperature:** a quantity in a scale unit (K, °C, °F) is a reading, a
  point on the scale. `Δ°C` is a difference. Reading − reading is a difference.
  Adding two readings errors when either is on an affine scale (`20 °C + 5 °C`
  suggests `Δ°C`). In products a reading counts as a difference, so
  `J/°C` and `4.18 J/(g °C) × 100 g × 10 °C` mean what they should.
- **Exactness:** `1/3 + 1/6` stays `1/2` until a float is forced.

## §4 Knowledge base

TOML under `data/`, compiled into the binary by `crates/kb/build.rs`, which
parses every value with the unit parser and checks it against its property's
declared dimension (`data/properties.toml`). Bad data fails the build with a
file and line.

```toml
[[entity]]
kind = "item"
display = "coffee"
names = ["coffee", "coffees", "cup of coffee"]
default = "caffeine"          # what `3 coffees` stands for
source = "USDA FoodData Central, …"
[entity.props]
caffeine = "95 mg"
volume = "8 floz"
```

- **Constants:** all 355 of CODATA 2022, with uncertainties
  (`scripts/data/codata.py`).
- **Elements:** all 118, from PubChem (`scripts/data/elements.py`).
- **Solar system:** the Sun, planets, Pluto and 21 moons, from NASA's NSSDCA
  fact sheets (`scripts/data/solar.py`).
- **Foods:** 43 household portions from USDA FoodData Central, SR Legacy
  (`scripts/data/food.py`): mass, energy, caffeine, sugar.
- **Everyday things:** 69 hand-written entries (sport, standards, coins,
  buildings, vehicles, nature), each citing its source, some tagged as
  **for-scale** references. Values that could not be checked were left out.

Every value carries a `source`; `w --why` shows it.

**Asking:** `mass of earth`, `earth's mass`, `earth.mass`; `3 coffees` stands for
the item's default property; `in g of caffeine` picks another; a lone name
(`gold`) shows its card. A name that is already a unit (`W`, `h`) stays the
unit unless `--as element` asks.

## §5 Pods

| Pod | Shown when |
|---|---|
| interpretation | always |
| assuming | a word had several plausible meanings |
| result | always |
| exact form | result is a fraction or involves π or √ |
| other units | result has a dimension; 3–4 units chosen by magnitude |
| for scale | result has a dimension with good reference items |
| uncertainty | any input carried `±` |
| solutions, steps | `solve` |
| plot | `plot`, or a bare `f(x)` definition |
| properties | functions (zeros, symmetry, limits), entities |
| table | `table f(x) for x in 0..10` |
| entity card | the query is just an entity name |

- **Other units:** each of the quantity's unit groups (metric, customary, …)
  offers the unit whose value reads best: 1–1000 is ideal, roughly 0.0003 to
  three million is acceptable. Extra groups (nautical, astronomical) speak only
  when the result is already in one of their units or nothing else fits. The SI
  unit always comes last. Temperatures always show every scale.
- **For scale:** the reference whose ratio is closest to a round number between
  ½ and 1000; at most two.

## §6 Output modes

```
w 5 mi in km            full pods
w -q 5 mi in km         8.04672  (result only, for scripts)
w --json 5 mi in km     structured: pods, value, unit, dimension
w --plain ...           no colour, ASCII boxes
echo "3 ft in cm" | w   stdin, one query per line
```

- Exact integers print in full (up to 12 digits as an answer, 6 as an
  alternative). Fractions stay fractions when every input was a whole number
  (`1/3 + 1/6` → `1/2`). Exact decimals print in full up to 12 significant
  digits (`8.04672`).
- Everything else rounds to 4 significant figures (`--sig N`), keeping integer
  digits below a million (`217 261`), and goes scientific outside
  0.001–999 999 (`1.813×10¹⁴`, or `1.813e14` under `--plain`).
- Integer parts of five or more digits are grouped in threes with a narrow
  no-break space, or commas under `--plain`.
- On entity cards, a value shown in its own unit keeps the significant digits
  its source published, trailing zeros included, grouped as CODATA writes
  them: G is `6.674 30×10⁻¹¹`, c is `299 792 458`. Input accepts either, and plain
  spaces too (`1 000 000`).

## §7 Errors

```
  ◆ can't add these
  │ 3 m + 2 s
  │ ─┬─   ─┬─
  │  │     └ time
  │  └ length
  │ hint: did you mean 3 m / 2 s  (= 1.5 m/s)?
```

Unknown words get an edit-distance suggestion: `unknown unit "kilometres"? → km`.

## §8 REPL

`w` with no arguments. Prompt `W›`. Live highlighting (numbers amber, units
cyan, entities green, unknown words underlined), tab completion, `it`, and
`:vars` `:clear` `:pods on|off` `:sig N` `:why`. History in
`~/.local/share/tungsten/history`.

## §9 Notebooks (`.w`)

Plain text evaluated top to bottom, variables carried forward:

```
cabinets = 4200 usd                          4 200 usd
counters = 38 sq ft * 85 usd/sq ft           3 230 usd
labour   = 60 h * 55 usd/h                   3 300 usd
total    = cabinets + counters + labour     10 730 usd
total * 1.1                                 11 803 usd
```

`w run file.w` prints results in a right-hand column. `--watch` re-evaluates on
save. `--write` stamps `#=> …` comments into the file, overwritten each run.

## §10 Dates and time

`days until 2026-12-25` · `2026-09-22 + 90 days` · `1 billion seconds in years`.
No time zones in the first versions.

## §11 Easter eggs

- `w 74` answers normally, then: `74 is tungsten's atomic number.`
- `w wolfram` shows tungsten's element card and the *Wolf Rahm* etymology.
- `w --about` is the element card as a version screen.

## §12 Architecture

```
crates/
  core/     lexer, resolver, Pratt parser, AST, evaluator
  units/    unit table, prefixes, dimension vectors, conversion
  kb/       data/*.toml + build.rs → phf tables
  pods/     one module per pod type
  render/   layout, number formatting, braille plots, theme
  cli/      clap, reedline REPL, notebook runner
data/       units.toml, constants.toml, elements.toml, items.toml …
tests/snap/ golden output snapshots (insta)
```

Targets: cold start under 5 ms, any query under 1 ms.

## §13 Roadmap

Lives in cairn: `cairn roadmap`, or the rendered [`ROADMAP.md`](../ROADMAP.md).

## §14 Decisions

Open questions are `decision` items: `cairn list --view decisions`. Settled:

- **Binary name** (0026): the binary is `tungsten`. Docs write `w`, which is
  `alias w=tungsten`; installing never shadows POSIX `w`.
- **Parsing strictness** (0025): the §2 grammar is strict. Anything outside it
  fails with a caret and a did-you-mean; the interpretation pod always shows how
  words were read. Failed dogfooding queries become expected-error snapshots,
  and recurring patterns are promoted into the grammar deliberately.
