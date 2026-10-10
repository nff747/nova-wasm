use nova_wasm::*;

#[test]
fn test_comparisons() {
    let mut vm = VirtualMachine::new(1, 100);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(10),
        Instruction::I32Const(20),
        Instruction::I32LtS, // 1
        Instruction::I32Const(5),
        Instruction::I32Const(5),
        Instruction::I32Eq, // 1
        Instruction::I32Add, // 2
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    assert_eq!(res, Some(Value::I32(2)));
}
