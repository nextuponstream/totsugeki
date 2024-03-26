# tournament-organiser-web

A web interface to manage brackets as a tournament organiser.

Current development efforts are on being able to share brackets with others.

## Comment about the tech stack

Current application is a vue app mounted statically onto an axum http server. I
use Vue because that's the only frontend framework I know.

What is needed:

* vue draggable (or something equivalent like sortablejs)

There does not seem to exist any equivalent in any rust UI frameworks (like
dioxus). Seeding through drag and drop in place feels like a necessary feature
for such an app.

* Tailwind

The only way I found to create a visualization of a bracket. Just say which rows
this thing go in the grid!

## Recommended IDE Setup

At the moment, I'd recommend RustRover because having the database tool with
typescript lsp setup out-of-the-box is invaluable.

If you cannot, then I'd recommend vscode (see [doc](../docs/vscode_setup.md)).

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

Then connect to http://localhost:5173

In the background, run an instance of the API:

```bash
DB_USERNAME=toa DB_PASSWORD=toa DB_NAME=toa cargo watch \
-w tournament-organiser-api -s "cd tournament-organiser-api && cargo run"
```

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

### Run Unit Tests with [Vitest](https://vitest.dev/)

```sh
npm run test:unit
```

### Run End-to-End Tests with [Cypress](https://www.cypress.io/)

First, run the application in the background

```bash
BUILD_PATH_TOURNAMENT_ORGANISER_WEB=tournament-organiser-web/dist \
DB_USERNAME=toa DB_PASSWORD=toa DB_NAME=toa \
cargo watch -w tournament-organiser-api \
-s "cargo run --package tournament-organiser-api"
```

Seed the database:

```bash
sudo -u toa psql --dbname=toa < e2eSeeder.sql
```

Launch interactive e2e cypress tests

```bash
npm run test:e2e
```

Optionally, if you want to be closer to the real production setup:

```bash
BUILD_PATH_TOURNAMENT_ORGANISER_WEB=tournament-organiser-web/dist \
DB_USERNAME=toa DB_PASSWORD=toa DB_NAME=toa \
cargo watch -w tournament-organiser-web -w tournament-organiser-api \
-s "npm --prefix tournament-organiser-web run build && cargo run --package tournament-organiser-api"
```

### Lint with [ESLint](https://eslint.org/)

```sh
npm run lint
```