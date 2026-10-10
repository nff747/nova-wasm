use nova_wasm::i64_ops::*;

#[test]
fn test_i64_bitwise_ops() {
    assert_eq!(i64_and(0xFF, 0x0F), 0x0F);
    assert_eq!(i64_or(0xF0, 0x0F), 0xFF);
    assert_eq!(i64_xor(0xFF, 0xFF), 0);
    assert_eq!(i64_shl(1, 10), 1024);
    assert_eq!(i64_shr_s(1024, 2), 256);
}
