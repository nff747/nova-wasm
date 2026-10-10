//! Executable bytecode instructions for Nova VM.

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Nop,
    Unreachable,
    // Constants
    I32Const(i32),
    I64Const(i64),
    // Local / Global
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    // Arithmetic I32
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    // Comparison I32
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    // Memory
    I32Load { offset: u32 },
    I32Store { offset: u32 },
    // Control
    Return,
    Call(u32),
}
