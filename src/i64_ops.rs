//! 64-bit integer arithmetic helper routines.

pub fn i64_add(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

pub fn i64_sub(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

pub fn i64_mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

pub fn i64_and(a: i64, b: i64) -> i64 {
    a & b
}

pub fn i64_or(a: i64, b: i64) -> i64 {
    a | b
}

pub fn i64_xor(a: i64, b: i64) -> i64 {
    a ^ b
}

pub fn i64_shl(a: i64, b: u32) -> i64 {
    a.wrapping_shl(b)
}

pub fn i64_shr_s(a: i64, b: u32) -> i64 {
    a.wrapping_shr(b)
}
