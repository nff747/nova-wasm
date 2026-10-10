//! Invocation frame and variable scope.

use crate::trap::Trap;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub func_idx: u32,
    pub pc: usize,
    pub locals: Vec<Value>,
}

impl CallFrame {
    pub fn new(func_idx: u32, locals: Vec<Value>) -> Self {
        Self {
            func_idx,
            pc: 0,
            locals,
        }
    }

    pub fn get_local(&self, idx: usize) -> Result<Value, Trap> {
        self.locals.get(idx).copied().ok_or(Trap::InvalidType)
    }

    pub fn set_local(&mut self, idx: usize, val: Value) -> Result<(), Trap> {
        if idx >= self.locals.len() {
            return Err(Trap::InvalidType);
        }
        self.locals[idx] = val;
        Ok(())
    }
}
