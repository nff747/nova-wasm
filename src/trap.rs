//! Runtime traps and execution exceptions in Nova-WASM.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trap {
    Unreachable,
    DivideByZero,
    IntegerOverflow,
    OutOfBoundsMemoryAccess,
    StackOverflow,
    StackUnderflow,
    OutOfGas,
    InvalidType,
    CallDepthExceeded,
    UndefinedElement,
}

impl std::fmt::Display for Trap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trap::Unreachable => write!(f, "trap: unreachable executed"),
            Trap::DivideByZero => write!(f, "trap: integer divide by zero"),
            Trap::IntegerOverflow => write!(f, "trap: integer overflow"),
            Trap::OutOfBoundsMemoryAccess => write!(f, "trap: out of bounds memory access"),
            Trap::StackOverflow => write!(f, "trap: value stack overflow"),
            Trap::StackUnderflow => write!(f, "trap: value stack underflow"),
            Trap::OutOfGas => write!(f, "trap: instruction gas budget exhausted"),
            Trap::InvalidType => write!(f, "trap: invalid value type encountered"),
            Trap::CallDepthExceeded => write!(f, "trap: maximum call depth exceeded"),
            Trap::UndefinedElement => write!(f, "trap: undefined table element"),
        }
    }
}

impl std::error::Error for Trap {}
