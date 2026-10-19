# A 6502 Emulator

A MOS 6502 emulator written in Rust, with a CLI runner, WebAssembly bindings, and a React frontend that drives the emulator in the browser.

## Features

- **6502 CPU core** (`src/processor.rs`) — full instruction table, status flags, cycle-by-cycle `clock()` execution, 64 KB memory map (16 KB RAM, 16 KB memory-mapped I/O/other, 32 KB ROM), reset vector at `0xFFFC`.
- **Bus / memory / ROM modules** (`src/bus.rs`, `src/memory.rs`, `src/rom.rs`) — load a ROM binary and run it for a fixed number of cycles.
- **CLI runner** (`src/main.rs`, bin `6502-emulator`) — loads `6502_functional_test.bin` from the current directory and executes up to 10,000,000 cycles (debug builds also print the instruction set table).
- **WASM library** (`src/wasm_interface.rs`) — exposes the processor to JavaScript via `wasm-bindgen`; a single processor instance is wrapped in a `RefCell` for global access, and storage layout is exposed to JS.
- **React frontend** (`react-frontend/`) — TypeScript app (Create React App + `react-app-rewired`) with pages for initialization, ROM loading, an overview of processor state, and disassembly.

## Project layout

```
src/                  Rust core + CLI
  processor.rs        6502 CPU
  bus.rs, memory.rs   64 KB memory map
  rom.rs              ROM loading
  wasm_interface.rs   wasm-bindgen exports (cdylib)
wasm-build.sh         wasm-pack build script
react-frontend/       React/TypeScript UI (depends on file:../wasm-6502)
```

## Build and run

### CLI

```sh
cargo run            # debug build; place 6502_functional_test.bin in the working directory
cargo test           # runs the wasm-bindgen tests
```

### WebAssembly + frontend

```sh
./wasm-build.sh                       # wasm-pack build --out-dir="./wasm-6502"
cd react-frontend
npm install                           # wasm-6502 is linked as file:../wasm-6502
npm start                             # dev server (react-app-rewired)
```

The release profile optimizes for code size (`opt-level = "s"`); the optional
`wee_alloc` and `console_error_panic_hook` features are enabled by default.

## References

- [6502.org CPU info](https://www.6502.org/)
- [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen)
- [wasm-pack](https://github.com/wasm-bindgen/wasm-pack)
