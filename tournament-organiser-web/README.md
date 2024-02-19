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

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur) + [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin).

This project uses eslint, use eslint vscode extension with the following editor
settings:

```json
    ...
    "editor.rulers": [80],
    "editor.codeActionsOnSave": {
        "source.fixAll": "never",
        "source.fixAll.eslint": "always"
    },
    "eslint.validate": [
        "javascript"
    ],
    "todohighlight.keywords": [
        {
            "text": "TODO",
            "color": "rgb(13, 184, 38)",
            "backgroundColor": "rgba(6, 89, 18,.4)",
            "isWholeLine": true,
        },
        {
            "text": "FIXME",
            "color": "rgb(242, 51, 51)",
            "backgroundColor": "rgba(138, 28, 28,.4)",
            "isWholeLine": true,
        },
        {
            "text": "NOTE",
            "color": "rgb(13, 184, 38)",
            "backgroundColor": "rgba(6, 89, 18,.1)",
            "isWholeLine": false,
        },
    ],
    ...
```

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
cargo watch -w tournament-organiser-web -w tournament-organiser-api \
-s "npm --prefix tournament-organiser-web run build && cargo run --package tournament-organiser-api"
```

Seed the database:

```bash
sudo -u toa psql --dbname=toa < e2eSeeder.sql
```

Launch interactive e2e cypress tests

```bash
npm run test:e2e
```

NOTE: I have not found a way to use `npm run dev` while the API runs on the same
port during cypress testing. This would be nice because you would benefit from hot-reloading rather than rebuilding the app completely during testing.
Currently, telling cypress to target something else than the API (API on 8080, 
dev on 5173) fails because registration test has a CORS problem (not same 
origin).

### Lint with [ESLint](https://eslint.org/)

```sh
npm run lint
```