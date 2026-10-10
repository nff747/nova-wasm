use nova_wasm::*;

#[test]
fn test_instance_function_invoke() {
    let vm = VirtualMachine::new(1, 1000);
    let mut instance = ModuleInstance::new(vm);

    let func = CompiledFunction::new(
        FuncType { params: vec![ValType::I32, ValType::I32], results: vec![ValType::I32] },
        0,
        vec![
            Instruction::LocalGet(0),
            Instruction::LocalGet(1),
            Instruction::I32Add,
            Instruction::Return,
        ],
    );

    let idx = instance.add_function(func);
    instance.export_function("add", idx);

    let res = instance.invoke("add", &[Value::I32(19), Value::I32(23)]).unwrap();
    assert_eq!(res, Some(Value::I32(42)));
}
