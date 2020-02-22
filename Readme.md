# s1vm

A WebAssembly (wasm) interpreter written in 100% safe Rust.

This project started with the idea to port [WASM3](https://github.com/wasm3/wasm3)'s
VM design to safe Rust.

See [Ideas.md](./Ideas.md) for some crazy ideas that might be used.

## Goals

1. Only use safe Rust.  Crate marked `#![forbid(unsafe_code)]`
2. Support pause/resume.  Either by using `async/await` or stack unwinding/rewinding.

