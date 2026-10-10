use nova_wasm::*;

#[test]
fn bench_one_million_instructions() {
    let mut vm = VirtualMachine::new(1, 2_000_000);
    let mut frame = CallFrame::new(0, vec![Value::I32(0)]);

    // Loop 100,000 times with 10 instructions per loop = 1M instructions
    let instrs = vec![
        Instruction::LocalGet(0),
        Instruction::I32Const(1),
        Instruction::I32Add,
        Instruction::LocalTee(0),
        Instruction::I32Const(100_000),
        Instruction::I32LtS,
        Instruction::BrIf(0),
        Instruction::LocalGet(0),
        Instruction::Return,
    ];

    let start = std::time::Instant::now();
    let res = vm.execute_instructions(&mut frame, &instrs).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(res, Some(Value::I32(100_000)));
    println!("100,000 iterations executed in {:?}", elapsed);
}
