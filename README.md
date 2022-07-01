# Totsugeki

Deploy an OpenAPI compliant backend that helps you manage tournaments with a
discord bot.

## Roadmap MVP

* [ ] let player !participate in bracket
* [ ] report !result
* [ ] let player see their !nextmatch opponent
* [ ] TO !validatematch
* [ ] !finalize bracket when tournament is over

## Installation

Create discord server and discord [bot](https://discord.com/developers/). Set
bot permissions:

* send messages
* read chat history

Invite discord bot to server. Build all binaries:

```bash
# frontend package requires environment variables to be set at build time:
# export $(xargs < .env)
cargo build --release --workspace
```

### Deploy infrastructure

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

Then deploy:

```bash
for s in $(ls src/bin); do cargo watch -x "run --bin ${s%.*}" &; done
```

### Run tests

Deploy a tournament server. Discord bot binary is not used in tests. Instead,
cucumber-rs is used to make the same API calls to the tournament server the
discord bot would have made.

**Note**: Set `TESTING` environment variable if you want the test
infrastructure to use `.env-test` instead of `.env`.

```bash
TESTING=1 cargo watch -x "run --bin tournament-server"
```

Open another terminal:

```bash
cargo watch -x test
```
