use nova_wasm::*;

#[test]
fn test_locals_get_set() {
    let mut vm = VirtualMachine::new(1, 100);
    let mut frame = CallFrame::new(0, vec![Value::I32(10), Value::I32(0)]);

    let instrs = vec![
        Instruction::LocalGet(0),
        Instruction::I32Const(5),
        Instruction::I32Add,
        Instruction::LocalTee(1),
        Instruction::I32Const(2),
        Instruction::I32Mul,
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    assert_eq!(res, Some(Value::I32(30)));
    assert_eq!(frame.get_local(1).unwrap(), Value::I32(15));
}
