# Totsugeki

Deploy an OpenAPI compliant backend that helps you manage tournaments with a
discord bot.

## Roadmap MVP

* [x] let player !join bracket
* [ ] report !result
* [ ] let player see their !nextmatch opponent
* [ ] TO !validatematch
* [ ] !finalize bracket when tournament is over

## Installation

Create discord server and discord [bot](https://discord.com/developers/). Set
bot permissions:

* send messages
* read chat history
* manage roles

Invite discord bot to server. Build all binaries:

```bash
# frontend package requires environment variables to be set at build time:
# export $(xargs < .env)
 cargo build-core && cargo build-frontend   
```

### Deploy locally

Create `.env` using `.env.example`. For development, generate self-signed
certificate:

```bash
openssl req -newkey rsa:4096 \
-x509 \
-sha256 \
-days 3650 \
-nodes \
-out dev.crt \
-keyout dev.key
```

Follow workspace members README to deploy and take a look at available
aliases (`cargo --list`).

## Developping

For easier development, use `cargo install cargo-watch' and use related
aliases.

### Checks

Because workspaces have different build targets, use the following command to
check project code.

```bash
cargo watch -x check-core -x check-frontend
```

### Run tests

```bash
cargo test
```
