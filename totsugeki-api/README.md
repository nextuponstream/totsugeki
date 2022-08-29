# Tournament server

Backend for discord bot and frontend service.

## Developping

Start a tournament server instance to test other services by running from
project root (to load `.env` variables):

```bash
cargo watch -x "run -r -p totsugeki-api"
```

### Pretty logging

By default, bunyan formatter is used. Install cli tool (`cargo install bunyan`),
then pipe logs:

```bash
cargo watch -x "run -r -p totsugeki-api" | bunyan
```

## Contributing

### Extend

Deploy tournament server binary and consult API documentation at
`API_ADDR:API_PORT/swagger` as defined in `.env`.
