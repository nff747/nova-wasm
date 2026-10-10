use nova_wasm::*;

#[test]
fn test_fibonacci_iterative() {
    let mut vm = VirtualMachine::new(1, 10000);
    // calculate fib(7) = 13
    // locals: [n=7, a=0, b=1, i=0]
    let mut frame = CallFrame::new(0, vec![
        Value::I32(7),
        Value::I32(0),
        Value::I32(1),
        Value::I32(0),
    ]);

    let instrs = vec![
        // 0: loop head check i < n
        Instruction::LocalGet(3),
        Instruction::LocalGet(0),
        Instruction::I32LtS,
        Instruction::I32Eqz,
        Instruction::BrIf(16), // exit if i >= n

        // temp = a + b
        Instruction::LocalGet(1),
        Instruction::LocalGet(2),
        Instruction::I32Add, // a + b

        // a = b
        Instruction::LocalGet(2),
        Instruction::LocalSet(1),

        // b = temp
        Instruction::LocalSet(2),

        // i++
        Instruction::LocalGet(3),
        Instruction::I32Const(1),
        Instruction::I32Add,
        Instruction::LocalSet(3),

        Instruction::Br(0),

        // 14: return a
        Instruction::LocalGet(1),
        Instruction::Return,
    ];

    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    assert_eq!(res, Some(Value::I32(13)));
}
