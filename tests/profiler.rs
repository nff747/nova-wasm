use nova_wasm::*;

#[test]
fn test_execution_profile_metrics() {
    let mut profile = ExecutionProfile::default();
    for _ in 0..100 {
        profile.record_instruction();
    }
    profile.record_memory_op(true);
    profile.record_memory_op(false);
    assert_eq!(profile.total_instructions, 100);
    assert_eq!(profile.memory_writes, 1);
    assert_eq!(profile.memory_reads, 1);
}
