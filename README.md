# actionslogs-rs

Parsing actions logs with rust 🦀

🔗 [Demo](https://logs.reb.gg/)

If you want something slower, there's [`robherley/actionslogs-js`](https://github.com/robherley/actionslogs-js/) too.

## Setup

Requires:

1. [`wasm-pack`](https://github.com/rustwasm/wasm-pack)
2. [`bun`](https://bun.sh/)
3. (optional) [`cargo-watch`](https://github.com/watchexec/cargo-watch)

## Scripts

### `script/build [--wasm|--js]`

Builds wasm and web assets.

### `script/server`

Starts web development server (vite) and will rebuild wasm if changes detected.

