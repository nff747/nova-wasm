//! Performance counter and instruction profiling statistics.

#[derive(Debug, Default, Clone)]
pub struct ExecutionProfile {
    pub total_instructions: u64,
    pub memory_reads: u64,
    pub memory_writes: u64,
    pub branches_taken: u64,
}

impl ExecutionProfile {
    pub fn record_instruction(&mut self) {
        self.total_instructions += 1;
    }

    pub fn record_memory_op(&mut self, write: bool) {
        if write {
            self.memory_writes += 1;
        } else {
            self.memory_reads += 1;
        }
    }
}
