//! Instantiated WebAssembly module container.

use crate::function::CompiledFunction;
use crate::trap::Trap;
use crate::value::Value;
use crate::vm::VirtualMachine;
use crate::frame::CallFrame;
use std::collections::HashMap;

pub struct ModuleInstance {
    pub vm: VirtualMachine,
    pub functions: Vec<CompiledFunction>,
    pub exports: HashMap<String, u32>,
}

impl ModuleInstance {
    pub fn new(vm: VirtualMachine) -> Self {
        Self {
            vm,
            functions: Vec::new(),
            exports: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, func: CompiledFunction) -> u32 {
        let idx = self.functions.len() as u32;
        self.functions.push(func);
        idx
    }

    pub fn export_function(&mut self, name: &str, func_idx: u32) {
        self.exports.insert(name.to_string(), func_idx);
    }

    pub fn invoke(&mut self, name: &str, args: &[Value]) -> Result<Option<Value>, Trap> {
        let func_idx = self.exports.get(name).copied().ok_or(Trap::UndefinedElement)?;
        let func = self.functions.get(func_idx as usize).ok_or(Trap::UndefinedElement)?.clone();

        let mut locals = Vec::with_capacity(args.len() + func.num_locals);
        locals.extend_from_slice(args);
        for _ in 0..func.num_locals {
            locals.push(Value::I32(0));
        }

        let mut frame = CallFrame::new(func_idx, locals);
        self.vm.execute_instructions(&mut frame, &func.instructions)
    }
}
