//! Execution operand value stack.

use crate::trap::Trap;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct ValueStack {
    data: Vec<Value>,
    max_depth: usize,
}

impl ValueStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            data: Vec::with_capacity(256),
            max_depth,
        }
    }

    pub fn push(&mut self, val: Value) -> Result<(), Trap> {
        if self.data.len() >= self.max_depth {
            return Err(Trap::StackOverflow);
        }
        self.data.push(val);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value, Trap> {
        self.data.pop().ok_or(Trap::StackUnderflow)
    }

    pub fn peek(&self) -> Result<&Value, Trap> {
        self.data.last().ok_or(Trap::StackUnderflow)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
