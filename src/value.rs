//! Strongly typed WebAssembly values.

use crate::trap::Trap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    pub fn as_i32(&self) -> Result<i32, Trap> {
        match self {
            Value::I32(v) => Ok(*v),
            _ => Err(Trap::InvalidType),
        }
    }

    pub fn as_i64(&self) -> Result<i64, Trap> {
        match self {
            Value::I64(v) => Ok(*v),
            _ => Err(Trap::InvalidType),
        }
    }

    pub fn as_f32(&self) -> Result<f32, Trap> {
        match self {
            Value::F32(v) => Ok(*v),
            _ => Err(Trap::InvalidType),
        }
    }

    pub fn as_f64(&self) -> Result<f64, Trap> {
        match self {
            Value::F64(v) => Ok(*v),
            _ => Err(Trap::InvalidType),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::I32(v) => *v != 0,
            Value::I64(v) => *v != 0,
            Value::F32(v) => *v != 0.0 && !v.is_nan(),
            Value::F64(v) => *v != 0.0 && !v.is_nan(),
        }
    }
}
