# Frontend

Website in web assembly.

## Usage

### Build

Use node lts: `nvm install --lts`

Api address and other environment variables are loaded at build time. Load
environment variable from `.env` file at the root of this repo:
`export $(xargs < .env)`

```bash
yarn install
yarn run build
yarn run dev --watch
```

### Dev setup

In chrome, paste `chrome://flags/#allow-insecure-localhost` to trust
self-signed certificates.

In firefox, add an exception for localhost:3000 under options > privacy and
security certificates (bottom) > view certificates > servers tab
