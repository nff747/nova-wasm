use nova_wasm::*;

#[test]
fn test_bitwise_ops() {
    let mut vm = VirtualMachine::new(1, 100);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(0b1100),
        Instruction::I32Const(0b1010),
        Instruction::I32And,
        Instruction::I32Const(0b0011),
        Instruction::I32Or,
        Instruction::I32Const(1),
        Instruction::I32Shl,
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    // (1100 & 1010) = 1000 (8). (1000 | 0011) = 1011 (11). 11 << 1 = 22.
    assert_eq!(res, Some(Value::I32(22)));
}
