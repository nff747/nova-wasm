use nova_wasm::*;

#[test]
fn test_memory_oob_trap() {
    let mut vm = VirtualMachine::new(1, 100);
    let mut frame = CallFrame::new(0, vec![]);

    let instrs = vec![
        Instruction::I32Const(70000), // outside 1 page (65536)
        Instruction::I32Load { offset: 0 },
    ];

    let err = vm.execute_instructions(&mut frame, &instrs).unwrap_err();
    assert_eq!(err, Trap::OutOfBoundsMemoryAccess);
}
