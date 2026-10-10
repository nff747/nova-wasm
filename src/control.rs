//! Control stack frames for block, loop and if structures.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Block,
    Loop,
    If,
}

#[derive(Debug, Clone)]
pub struct ControlFrame {
    pub block_type: BlockType,
    pub target_pc: usize,
    pub stack_size: usize,
}

impl ControlFrame {
    pub fn new(block_type: BlockType, target_pc: usize, stack_size: usize) -> Self {
        Self {
            block_type,
            target_pc,
            stack_size,
        }
    }
}
