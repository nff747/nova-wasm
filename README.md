# ⚡ nova-wasm

> Zero-copy WebAssembly binary format decoder and LEB128 streaming parser in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org/)

`nova-wasm` provides a low-overhead, memory-efficient WebAssembly binary parser engineered for edge runtimes, static analyzers, and bytecode inspectors.

---

## 🚀 Features

- **Zero-Copy Parsing**: Operates directly over raw byte slices without heap allocations.
- **Fast LEB128 Decoding**: Highly optimized branchless signed and unsigned LEB128 variable-length integer decoders.
- **WASM Section Decoder**: Complete section header, export table, type signature, and opcode stream decoding.
- **Validation**: Strict WASM magic byte (`\0asm`) and specification compliance verification.

---

## 🧪 Testing & Build

```bash
cargo test
cargo build --release
```

---

## 📄 License

MIT © [nff747](https://github.com/nff747)
