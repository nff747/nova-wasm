use nova_wasm::*;

#[test]
fn test_vm_addition_and_multiplication() {
    let mut vm = VirtualMachine::new(1, 1000);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(21),
        Instruction::I32Const(2),
        Instruction::I32Mul,
        Instruction::I32Const(8),
        Instruction::I32Add,
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    assert_eq!(res, Some(Value::I32(50)));
}
