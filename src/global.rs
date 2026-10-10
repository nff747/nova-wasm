//! WebAssembly global variable instances.

use crate::trap::Trap;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct GlobalVariable {
    val: Value,
    mutable: bool,
}

impl GlobalVariable {
    pub fn new(val: Value, mutable: bool) -> Self {
        Self { val, mutable }
    }

    pub fn get(&self) -> Value {
        self.val
    }

    pub fn set(&mut self, new_val: Value) -> Result<(), Trap> {
        if !self.mutable {
            return Err(Trap::InvalidType);
        }
        self.val = new_val;
        Ok(())
    }
}
