# Totsugeki

Library to create and manage competitions and tournaments. The language can get
confusing but for the purpose of this library, we define:

- a tournament is a single competition, like a single elimination tournament
- a participant plays in a tournament
- an event is a collection of tournaments
- an attendee is a person in an event that plays in at least one tournament
- a bracket runner is helps running a tournament
- a bracket runner can be a participant
- a user is a person
- a tournament has many matches
- matches have 2 players at most

## Examples

Check out `examples` directory. A ready-made discord bot is available in
`../totsugeki-discord-bot`.

## Tests

### coverage report

```bash
# install llvm-cov
cargo +stable install cargo-llvm-cov --locked
# runs cargo test and reports coverage
cargo llvm-cov --html --open
```
