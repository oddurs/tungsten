# Fuzzing

One target, `query`, pushes arbitrary UTF-8 through the whole pipeline: lex,
resolve, parse, check, evaluate, build pods, and render at two widths in both
fancy and plain modes. Nothing may panic.

```sh
# seed from the snapshot corpus (the corpus directory is not committed)
mkdir -p fuzz/corpus/query
grep -v '^#' tests/queries.txt | grep -v '^$' | split -l 1 - fuzz/corpus/query/seed-

cd fuzz && cargo +nightly fuzz run query -- -max_total_time=600 -max_len=128
```

A crash leaves its input in `fuzz/artifacts/query/`. Reproduce it with
`cargo +nightly fuzz run query fuzz/artifacts/query/<file>`, then add the
query to `tests/queries.txt` once it is fixed.
