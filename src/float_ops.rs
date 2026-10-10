//! IEEE 754 floating-point bitcasting and classification.

pub fn f32_to_bits(f: f32) -> u32 {
    f.to_bits()
}

pub fn f32_from_bits(b: u32) -> f32 {
    f32::from_bits(b)
}

pub fn f64_to_bits(f: f64) -> u64 {
    f.to_bits()
}

pub fn f64_from_bits(b: u64) -> f64 {
    f64::from_bits(b)
}
