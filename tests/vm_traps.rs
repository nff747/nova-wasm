use nova_wasm::*;

#[test]
fn test_divide_by_zero_trap() {
    let mut vm = VirtualMachine::new(1, 100);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(100),
        Instruction::I32Const(0),
        Instruction::I32DivS,
    ];

    let err = vm.execute_instructions(&mut frame, &instrs).unwrap_err();
    assert_eq!(err, Trap::DivideByZero);
}
