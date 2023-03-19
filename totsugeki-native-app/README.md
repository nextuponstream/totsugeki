# Native App

An experimental app made with dioxus, tailwind and totsugeki library.

**IMPORTANT**: current app can display up to 16384 players. If you wish to go
beyond that limit, you need to regenerate the corresponding css.

## Developement

Install tailwind CLI:

```bash
npm i -g tailwindcss
```

Generate css: 

```bash
cargo run --bin css_row.rs # edit src/bin/css_row.rs to increase player cap,
# then tailwind.config.js
tailwindcss -o ./resources/tailwind.css
```

Then you can serve with some hot-reload capability:

```bash
cargo run
```
