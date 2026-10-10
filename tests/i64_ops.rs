use nova_wasm::i64_ops::*;

#[test]
fn test_i64_arithmetic() {
    assert_eq!(i64_add(100, 200), 300);
    assert_eq!(i64_sub(500, 200), 300);
    assert_eq!(i64_mul(20, 15), 300);
}
