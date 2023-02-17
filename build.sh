#!/usr/bin/env bash
cargo build --release --target wasm32-unknown-unknown && \
wasm-bindgen target/wasm32-unknown-unknown/release/rougelike.wasm --out-dir wasm --no-modules --no-typescript
