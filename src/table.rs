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

    pub fn max_elements(&self) -> Option<usize> {
        self.max_elements
    }

    /// Grows the table by `delta` elements, up to optional `max_elements`.
    pub fn grow(&mut self, delta: usize, init_val: Option<u32>) -> Result<usize, Trap> {
        let prev_len = self.elements.len();
        let new_len = prev_len.checked_add(delta).ok_or(Trap::OutOfBoundsMemoryAccess)?;
        if let Some(max) = self.max_elements {
            if new_len > max {
                return Err(Trap::OutOfBoundsMemoryAccess);
            }
        }
        self.elements.resize(new_len, init_val);
        Ok(prev_len)
    }
}
