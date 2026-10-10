//! Nova-WASM Virtual Machine stack execution loop.

use crate::frame::CallFrame;
use crate::global::GlobalVariable;
use crate::instruction::Instruction;
use crate::memory::LinearMemory;
use crate::metering::GasMeter;
use crate::stack::ValueStack;
use crate::trap::Trap;
use crate::value::Value;

pub struct VirtualMachine {
    pub stack: ValueStack,
    pub memory: LinearMemory,
    pub globals: Vec<GlobalVariable>,
    pub gas_meter: GasMeter,
}

impl VirtualMachine {
    pub fn new(initial_memory_pages: usize, gas_limit: u64) -> Self {
        Self {
            stack: ValueStack::new(4096),
            memory: LinearMemory::new(initial_memory_pages, Some(256)),
            globals: Vec::new(),
            gas_meter: GasMeter::new(gas_limit),
        }
    }

    pub fn execute_instructions(&mut self, frame: &mut CallFrame, instructions: &[Instruction]) -> Result<Option<Value>, Trap> {
        while frame.pc < instructions.len() {
            self.gas_meter.consume(1)?;
            let instr = &instructions[frame.pc];
            frame.pc += 1;

            match instr {
                Instruction::Nop => {}
                Instruction::Unreachable => return Err(Trap::Unreachable),
                Instruction::I32Const(c) => self.stack.push(Value::I32(*c))?,
                Instruction::I64Const(c) => self.stack.push(Value::I64(*c))?,
                Instruction::LocalGet(idx) => {
                    let val = frame.get_local(*idx as usize)?;
                    self.stack.push(val)?;
                }
                Instruction::LocalSet(idx) => {
                    let val = self.stack.pop()?;
                    frame.set_local(*idx as usize, val)?;
                }
                Instruction::LocalTee(idx) => {
                    let val = *self.stack.peek()?;
                    frame.set_local(*idx as usize, val)?;
                }
                Instruction::GlobalGet(idx) => {
                    let g = self.globals.get(*idx as usize).ok_or(Trap::InvalidType)?;
                    self.stack.push(g.get())?;
                }
                Instruction::GlobalSet(idx) => {
                    let val = self.stack.pop()?;
                    let g = self.globals.get_mut(*idx as usize).ok_or(Trap::InvalidType)?;
                    g.set(val)?;
                }
                Instruction::I32Add => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a.wrapping_add(b)))?;
                }
                Instruction::I32Sub => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a.wrapping_sub(b)))?;
                }
                Instruction::I32Mul => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a.wrapping_mul(b)))?;
                }
                Instruction::I32DivS => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    if b == 0 {
                        return Err(Trap::DivideByZero);
                    }
                    if a == i32::MIN && b == -1 {
                        return Err(Trap::IntegerOverflow);
                    }
                    self.stack.push(Value::I32(a / b))?;
                }
                Instruction::I32DivU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    if b == 0 {
                        return Err(Trap::DivideByZero);
                    }
                    self.stack.push(Value::I32((a / b) as i32))?;
                }
                Instruction::I32RemS => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    if b == 0 {
                        return Err(Trap::DivideByZero);
                    }
                    if a == i32::MIN && b == -1 {
                        self.stack.push(Value::I32(0))?;
                    } else {
                        self.stack.push(Value::I32(a % b))?;
                    }
                }
                Instruction::I32RemU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    if b == 0 {
                        return Err(Trap::DivideByZero);
                    }
                    self.stack.push(Value::I32((a % b) as i32))?;
                }
                Instruction::I32And => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a & b))?;
                }
                Instruction::I32Or => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a | b))?;
                }
                Instruction::I32Xor => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a ^ b))?;
                }
                Instruction::I32Shl => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a.wrapping_shl(b)))?;
                }
                Instruction::I32ShrS => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(a.wrapping_shr(b)))?;
                }
                Instruction::I32ShrU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    self.stack.push(Value::I32((a >> (b % 32)) as i32))?;
                }
                Instruction::I32Eqz => {
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a == 0 { 1 } else { 0 }))?;
                }
                Instruction::I32Eq => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a == b { 1 } else { 0 }))?;
                }
                Instruction::I32Ne => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a != b { 1 } else { 0 }))?;
                }
                Instruction::I32LtS => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a < b { 1 } else { 0 }))?;
                }
                Instruction::I32LtU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    self.stack.push(Value::I32(if a < b { 1 } else { 0 }))?;
                }
                Instruction::I32GtS => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a > b { 1 } else { 0 }))?;
                }
                Instruction::I32GtU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    self.stack.push(Value::I32(if a > b { 1 } else { 0 }))?;
                }
                Instruction::I32LeS => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a <= b { 1 } else { 0 }))?;
                }
                Instruction::I32LeU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    self.stack.push(Value::I32(if a <= b { 1 } else { 0 }))?;
                }
                Instruction::I32GeS => {
                    let b = self.stack.pop()?.as_i32()?;
                    let a = self.stack.pop()?.as_i32()?;
                    self.stack.push(Value::I32(if a >= b { 1 } else { 0 }))?;
                }
                Instruction::I32GeU => {
                    let b = self.stack.pop()?.as_i32()? as u32;
                    let a = self.stack.pop()?.as_i32()? as u32;
                    self.stack.push(Value::I32(if a >= b { 1 } else { 0 }))?;
                }
                Instruction::I32Load { offset } => {
                    let addr = self.stack.pop()?.as_i32()? as usize;
                    let effective = addr + (*offset as usize);
                    let val = self.memory.read_i32(effective)?;
                    self.stack.push(Value::I32(val))?;
                }
                Instruction::I32Store { offset } => {
                    let val = self.stack.pop()?.as_i32()?;
                    let addr = self.stack.pop()?.as_i32()? as usize;
                    let effective = addr + (*offset as usize);
                    self.memory.write_i32(effective, val)?;
                }
                Instruction::Return => {
                    return Ok(self.stack.pop().ok());
                }
                Instruction::Call(_) => {
                    // Handled in instance execution
                }
                _ => {}
            }
        }
        Ok(self.stack.pop().ok())
    }
}
