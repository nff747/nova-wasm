//! WebAssembly table instance for funcref indirect calls.

use crate::trap::Trap;

#[derive(Debug, Clone)]
pub struct TableInstance {
    elements: Vec<Option<u32>>,
    max_elements: Option<usize>,
}

impl TableInstance {
    pub fn new(initial_size: usize, max_elements: Option<usize>) -> Self {
        Self {
            elements: vec![None; initial_size],
            max_elements,
        }
    }

    pub fn get(&self, idx: usize) -> Result<u32, Trap> {
        self.elements
            .get(idx)
            .copied()
            .flatten()
            .ok_or(Trap::UndefinedElement)
    }

    pub fn set(&mut self, idx: usize, func_idx: Option<u32>) -> Result<(), Trap> {
        if idx >= self.elements.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        self.elements[idx] = func_idx;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}
