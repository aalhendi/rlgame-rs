``cargo build --release --target wasm32-unknown-unknown
``
``wasm-bindgen target\wasm32-unknown-unknown\release\yourproject.wasm --out-dir wasm --no-modules --no-typescript``

Start a webserver in the ``wasm/`` directory. (i.e. ``python3 -m http.server``)
