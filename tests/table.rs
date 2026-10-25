use nova_wasm::*;

#[test]
fn test_table_elements() {
    let mut table = TableInstance::new(10, Some(20));
    assert_eq!(table.len(), 10);
    assert_eq!(table.get(0).unwrap_err(), Trap::UndefinedElement);

    table.set(3, Some(42)).unwrap();
    assert_eq!(table.get(3).unwrap(), 42);
}

#[test]
fn test_table_grow_and_max_boundary() {
    let mut table = TableInstance::new(5, Some(10));
    assert_eq!(table.max_elements(), Some(10));

    // Grow by 3 elements initialized with function index 77
    let old_len = table.grow(3, Some(77)).unwrap();
    assert_eq!(old_len, 5);
    assert_eq!(table.len(), 8);
    assert_eq!(table.get(6).unwrap(), 77);

    // Grow by 2 to reach exactly max limit (10)
    assert!(table.grow(2, None).is_ok());
    assert_eq!(table.len(), 10);

    // Exceed max limit -> Trap
    assert_eq!(table.grow(1, None).unwrap_err(), Trap::OutOfBoundsMemoryAccess);
}
