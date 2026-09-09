# Contributing to Nova

Thank you for your interest in contributing to **Nova**, a custom statically typed systems language compiling directly to WebAssembly!

## Development Setup

Nova requires a standard stable Rust toolchain:

```bash
cargo build
cargo test
```

### Running the CLI Compiler

```bash
cargo run --bin novac -- compile examples/fibonacci.nova -o fibonacci.wasm
```

## How to Submit Contributions

1. **Fork the repository** on GitHub.
2. **Create a topic branch**:
   ```bash
   git checkout -b feat/your-feature
   ```
3. **Commit your changes**:
   - Write unit tests in `tests/` covering new syntax, typechecker rules, or Wasm bytecode generation.
   - Verify all tests pass with `cargo test`.
4. **Submit a Pull Request** against `main`.

## License

By contributing to Nova, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
