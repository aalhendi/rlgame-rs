```sh
rustup target add wasm32-unknown-unknown
```

```sh
cargo install wasm-bindgen-cli
```

```sh
cargo build --release --target wasm32-unknown-unknown
```

```sh
wasm-bindgen target\wasm32-unknown-unknown\release\rougelike.wasm --out-dir wasm --no-modules --no-typescript
```

UNIX: Can also run ``./build.sh``
Start a webserver in the ``wasm/`` directory. (i.e. ``python3 -m http.server``)
