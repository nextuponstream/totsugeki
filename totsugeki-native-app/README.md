# Native App

An experimental app made with dioxus, tailwind and totsugeki library.

## Developement

Install tailwind CLI:

```bash
npm i -g tailwindcss
```

Generate css: 

```bash
cargo run --bin css_row.rs # edit tailwind.config.js
tailwindcss -o ./resources/tailwind.css
```

Then you can serve with some hot-reload capability:

```bash
cargo run
```
