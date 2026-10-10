//! Nova-WASM: Zero-copy, high-throughput WebAssembly binary format decoder, AST validator & execution runtime.

pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d]; // \0asm
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

pub mod control;
pub mod float_ops;
pub mod frame;
pub mod function;
pub mod global;
pub mod host;
pub mod i64_ops;
pub mod instruction;
pub mod instance;
pub mod memory;
pub mod metering;
pub mod stack;
pub mod trap;
pub mod value;
pub mod vm;

pub use control::{BlockType, ControlFrame};
pub use frame::CallFrame;
pub use function::CompiledFunction;
pub use global::GlobalVariable;
pub use host::HostRegistry;
pub use instruction::Instruction;
pub use instance::ModuleInstance;
pub use memory::LinearMemory;
pub use metering::GasMeter;
pub use stack::ValueStack;
pub use trap::Trap;
pub use value::Value;
pub use vm::VirtualMachine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
    V128 = 0x7B,
    FuncRef = 0x70,
    ExternRef = 0x6F,
}

impl ValType {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x7F => Some(ValType::I32),
            0x7E => Some(ValType::I64),
            0x7D => Some(ValType::F32),
            0x7C => Some(ValType::F64),
            0x7B => Some(ValType::V128),
            0x70 => Some(ValType::FuncRef),
            0x6F => Some(ValType::ExternRef),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportEntry {
    pub name: String,
    pub kind: ExportKind,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Func = 0x00,
    Table = 0x01,
    Memory = 0x02,
    Global = 0x03,
}

impl ExportKind {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(ExportKind::Func),
            0x01 => Some(ExportKind::Table),
            0x02 => Some(ExportKind::Memory),
            0x03 => Some(ExportKind::Global),
            _ => None,
        }
    }
}
