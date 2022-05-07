# wasi-cli

wasm standalone for PngDecoder CLI. Only validates an input PNG.
[If you want to embed the wasm into your native application (Rust, C, Python...)](https://docs.wasmtime.dev/lang.html), use `wasi-any` and not this one.

# how to do it
1. Install wasmtime 
```
curl https://wasmtime.dev/install.sh -sSf | bash
```

2. Add rustup as target
```
rustup target add wasm32-wasi
```

3. Compile
```
rustc src/main.rs --target wasm32-wasi
```

4. Run
```
wasmtime main.wasm
```

