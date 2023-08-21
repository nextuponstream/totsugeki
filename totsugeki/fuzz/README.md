# Fuzzing Totsugeki

Fuzzing requires the nightly toolchain (for now). If you are using the mold
linker, you may need to turn it off while fuzzing:

comment out `$HOME/.cargo/config.toml` the mold section.

## Run coverage tools

[source](https://rust-fuzz.github.io/book/cargo-fuzz/coverage.html)

Install llvm-tool for the nightly toolchain:

```bash
cargo install cargo-binutils # to call llvm-tools
rustup toolchain install nightly --component llvm-tools-preview
cargo cov --help
```

Fuzz and generate report:

```bash
cargo +nightly fuzz run big_brackets
cargo +nightly fuzz coverage big_brackets
# Show coverage for selected source code directory
cargo +nightly cov --verbose \                
-- show target/x86_64-unknown-linux-gnu/release/big_brackets \
--format html \
-instr-profile=coverage/big_brackets/coverage.profdata \  
path/to/repo/totsugeki \  
> index.html

xdg-open index.html
```

## Motives for fuzz targets

1. bracket

Test happy path and big brackets. Resolves winner, loser bracket, then grand
finals + grand final reset.

2. randomly ordered events

Resolving first winner, then loser bracket works but testing for unsoundness
may prove useful.

3. big brackets
Previous fuzz targets is stuck between 3 and 10 players. Also, first iterations
are stuck in single elimination bracket.

4. big brackets double elimination
Fuzzing thoroughly does not scale for 5000 (or even 555) players per iteration,
where 20 iterations might take an entire day. This is clearly too slow for such
combinatorial complexity. A lower number is taken.

5. very big brackets double elimination

Player brackets for giants events might go up to 7000+ players (evo 2023 SF6).
Fuzzing thoroughly through that many players is very slow.

```bash
cargo +nightly fuzz run very_big_brackets_de -- -timeout=50000
```

## About the code under test

There are guard rails in the code in the form of assertions. If the fuzz target
finds a way to break them, then that's a new unit test for regression suite.

Example: match where both players are the same player is malformed in the
context of single+double elimination format.

## Troubleshooting

### Oh no, the nightly build does not work anymore

Your nightly toolchain got updated with `rustup update` and Cargo.lock cannot
work with it (because it worked with an older version). Remove the lock file
and rerun fuzzing command.
