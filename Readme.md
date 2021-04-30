# s1vm

A fast WebAssembly (wasm) interpreter written in 100% safe Rust.

This project started with the idea to port [WASM3](https://github.com/wasm3/wasm3)'s
VM design to safe Rust.

See [Ideas.md](./Ideas.md) for some crazy ideas that might be used.

## Goals

1. Only use safe Rust.  Crate marked `#![forbid(unsafe_code)]`
2. Support pause/resume.  Either by using `async/await` or stack unwinding/rewinding.
3. Resource limits (CPU/Memory).  Limiting or throttling CPU usage is useful for running sandboxed code.

## Benchmark

Benchmark of `s1vm` against other wasm interpreters:
- [WASM3](https://github.com/wasm3/wasm3) - C
- [wasmi](https://github.com/paritytech/wasmi) - Rust

- fib.wasm 35
  - wasm3 - 0.66 seconds
  - s1vm  - 1.29 seconds
	- wasmi - 3.31 seconds

- fib.wasm 41
  - wasm3 -  9.8 seconds
  - s1vm  - 22.5 seconds
	- wasmi - 57.6 seconds

## TODOs

- [ ] - Support calling host functions.
- [ ] src/function.rs - Implement missing opcodes.
- [ ] src/compiler.rs - Implement missing compiler opcodes.
