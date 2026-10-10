use nova_wasm::*;

#[test]
fn test_host_functions() {
    let mut host = HostRegistry::new();
    host.register("env", "add_two", |args| {
        let a = args[0].as_i32()?;
        let b = args[1].as_i32()?;
        Ok(vec![Value::I32(a + b)])
    });

    let res = host.call("env", "add_two", &[Value::I32(40), Value::I32(2)]).unwrap();
    assert_eq!(res, vec![Value::I32(42)]);
}
