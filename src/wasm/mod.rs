pub mod binary_emitter;
pub mod leb128;
pub mod opcodes;
pub mod wat_emitter;

pub use binary_emitter::WasmBinaryEmitter;
pub use wat_emitter::WatEmitter;
