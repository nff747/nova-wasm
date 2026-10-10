use nova_wasm::*;

#[test]
fn test_table_elements() {
    let mut table = TableInstance::new(10, Some(20));
    assert_eq!(table.len(), 10);
    assert_eq!(table.get(0).unwrap_err(), Trap::UndefinedElement);

    table.set(3, Some(42)).unwrap();
    assert_eq!(table.get(3).unwrap(), 42);
}
