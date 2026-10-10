use nova_wasm::*;

#[test]
fn test_branching_loop() {
    let mut vm = VirtualMachine::new(1, 1000);
    // count up from 0 to 5
    let mut frame = CallFrame::new(0, vec![Value::I32(0)]);

    let instrs = vec![
        // pc 0: loop head
        Instruction::LocalGet(0),
        Instruction::I32Const(1),
        Instruction::I32Add,
        Instruction::LocalTee(0),
        Instruction::I32Const(5),
        Instruction::I32LtS, // true while < 5
        Instruction::BrIf(0), // jump back to 0 if true
        Instruction::LocalGet(0),
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    assert_eq!(res, Some(Value::I32(5)));
}
