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
