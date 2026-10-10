use nova_wasm::*;

#[test]
fn test_gas_exhaustion() {
    let mut vm = VirtualMachine::new(1, 3);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(1),
        Instruction::I32Const(2),
        Instruction::I32Add,
        Instruction::I32Const(3), // 4th instruction exceeds 3 gas
        Instruction::I32Add,
    ];

    let err = vm.execute_instructions(&mut frame, &instrs).unwrap_err();
    assert_eq!(err, Trap::OutOfGas);
}
