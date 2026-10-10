use nova_wasm::float_ops::*;

#[test]
fn test_float_bits() {
    let f = 3.14159f32;
    let b = f32_to_bits(f);
    assert_eq!(f32_from_bits(b), f);

    let d = 2.718281828459045f64;
    let db = f64_to_bits(d);
    assert_eq!(f64_from_bits(db), d);
}
