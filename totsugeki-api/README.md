# Tournament server

Backend for discord bot and frontend service.

## Developping

Start a tournament server instance to test other services by running from
project root (to load `.env` variables):

```bash
cd ..
cargo watch -x "run -p totsugeki-api"
```

When testing authenticated API calls, use the following authorization header:

```bash
cargo r --release -p totsugeki-api --bin api-token-generator -- --server-key <SERVER_KEY>

# one liner to generate secret
echo -n $(cargo r --release -p totsugeki-api --bin api-token-generator -- --server-key <SERVER_KEY>) > \
/path/to/totsugeki_api_token_for_discord_bot.txt
```

## Contributing

### Extend

Deploy tournament server binary and consult API documentation at
`API_ADDR:API_PORT/swagger` as defined in `.env`.
