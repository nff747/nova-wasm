# Nova-WASM Virtual Machine Execution Engine

## Overview
Nova-WASM has evolved from a pure binary AST decoder into a complete, sandboxed WebAssembly execution virtual machine written in Rust.

## Architecture
1. **Value Stack (`ValueStack`)**: Pre-allocated operand stack with O(1) push/pop and depth limits.
2. **Linear Memory (`LinearMemory`)**: Page-aligned (64KiB) virtual memory supporting bulk memory copy/fill and bounds checking.
3. **Instruction Metering (`GasMeter`)**: Strict instruction budget enforcement preventing runaway infinite loops.
4. **Host Dispatcher (`HostRegistry`)**: FFI binding layer allowing native host Rust functions to be imported seamlessly.
