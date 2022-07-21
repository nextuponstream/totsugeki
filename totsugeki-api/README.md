# Tournament server

Backend for discord bot and frontend service.

## Developping

Start a tournament server instance to test other services by running from
project root (to load `.env` variables):

```bash
cd ..
cargo watch -x "run -r -p totsugeki-api"
```

## Contributing

### Extend

Deploy tournament server binary and consult API documentation at
`API_ADDR:API_PORT/swagger` as defined in `.env`.
