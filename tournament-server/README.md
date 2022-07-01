# Tournament server

Backend for discord bot and frontend service.

## Developping

Start a tournament server instance to test other services by running from
project root (to load `.env` variables):

```bash
cd ..
cargo watch -x "run -p tournament-server"
```

## Contributing

### Extend

Deploy tournament server binary and consult API documentation at
`TOURNAMENT_SERVER_ADDR:TOURNAMENT_SERVER_PORT/swagger` as defined in `.env`.
