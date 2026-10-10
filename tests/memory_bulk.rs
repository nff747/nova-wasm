use nova_wasm::*;

#[test]
fn test_memory_fill_and_copy() {
    let mut mem = LinearMemory::new(1, None);
    mem.memory_fill(0, 0xAA, 16).unwrap();
    assert_eq!(mem.read_u8(0).unwrap(), 0xAA);
    assert_eq!(mem.read_u8(15).unwrap(), 0xAA);

    mem.memory_copy(16, 0, 16).unwrap();
    assert_eq!(mem.read_u8(16).unwrap(), 0xAA);
    assert_eq!(mem.read_u8(31).unwrap(), 0xAA);
}
