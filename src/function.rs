//! Compiled function structures and registry.

use crate::instruction::Instruction;
use crate::FuncType;

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub func_type: FuncType,
    pub num_locals: usize,
    pub instructions: Vec<Instruction>,
}

impl CompiledFunction {
    pub fn new(func_type: FuncType, num_locals: usize, instructions: Vec<Instruction>) -> Self {
        Self {
            func_type,
            num_locals,
            instructions,
        }
    }
}
