# Flyio hosting

What commands were used (around november).

Get the fly cli tool, please use it responsibly (dangerous)

```bash
curl -L https://fly.io/install.sh | sh
```

Sign in, enter your credit card info, yadda yadda

```bash
fly auth signup
```

Configure your fly.toml

```bash
hx fly.toml
```

Create your dockerfile (see Dockerfile) and upload that dockerfile to flyio.
Use that command only once:

```bash
fly launch -e BUILD_PATH_TOURNAMENT_ORGANISER_WEB=dist -e DOCKER_BUILD=1 \
-e PORT=8080 --region=cdg
```

For test, edit settings with

* name: totsugekitest
* port: 3000:
* memory: 256MB
* postgres: development

## Migrations

The app will not work because you need to run migration. Let's proxy commands
to the postgres dev instance:

```bash
fly proxy 5432 -a totsugekitest-pg -b localhost
```

TODO should provide command without needing to stop local postgresql service
with `sudo systemctl stop postgresql` because of occupied 5432 port.

Update your `tournament-organiser-api/.env` with
`DATABASE_URL=postgres://postgres:<PASSWORD>@localhost:5432/totsugekitest`

Then run migrations with run `cargo sqlx migrate run`

Verify that migrations (need proxy command to be active) where effectively ran
with: 

```bash
psql postgres://postgres:<PASSWORD>@localhost:5432/totsugekitest
# list databases
\l
# connect to totsugekitest
\c totsugekitest
# list tables
\d
# list columns to see new/removed columns from some table
SELECT * from users;
```

View your app in the browser to see the result.

## Other deployment

For all subsequent updates, do:

```bash
fly deploy
```

## Troubleshooting

WARN Failed to start remote builder heartbeat: failed building options: failed probing "personal": context deadline exceeded

Error: failed to fetch an image or build from source: error connecting to docker: failed building options: failed probing "personal": context deadline exceeded
➜  totsugeki git:(59-signup) ✗ fly wireguard reset
