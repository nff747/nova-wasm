use nova_wasm::*;

#[test]
fn test_memory_load_store() {
    let mut vm = VirtualMachine::new(1, 500);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(64), // address
        Instruction::I32Const(1337), // value
        Instruction::I32Store { offset: 0 },
        Instruction::I32Const(64),
        Instruction::I32Load { offset: 0 },
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    assert_eq!(res, Some(Value::I32(1337)));
}
