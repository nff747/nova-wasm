//! WebAssembly linear memory page allocation and safe byte reading.

use crate::trap::Trap;

pub const PAGE_SIZE: usize = 65536; // 64 KiB
pub const MAX_PAGES: usize = 65536; // 4 GiB theoretical max

#[derive(Debug, Clone)]
pub struct LinearMemory {
    bytes: Vec<u8>,
    max_pages: usize,
}

impl LinearMemory {
    pub fn new(initial_pages: usize, max_pages: Option<usize>) -> Self {
        let max_p = max_pages.unwrap_or(MAX_PAGES);
        let byte_len = initial_pages * PAGE_SIZE;
        Self {
            bytes: vec![0u8; byte_len],
            max_pages: max_p,
        }
    }

    pub fn pages(&self) -> usize {
        self.bytes.len() / PAGE_SIZE
    }

    pub fn grow(&mut self, additional_pages: usize) -> i32 {
        let current_pages = self.pages();
        let new_pages = current_pages + additional_pages;
        if new_pages > self.max_pages {
            return -1;
        }

        self.bytes.resize(new_pages * PAGE_SIZE, 0);
        current_pages as i32
    }

    pub fn read_u8(&self, offset: usize) -> Result<u8, Trap> {
        self.bytes.get(offset).copied().ok_or(Trap::OutOfBoundsMemoryAccess)
    }

    pub fn write_u8(&mut self, offset: usize, val: u8) -> Result<(), Trap> {
        if offset >= self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        self.bytes[offset] = val;
        Ok(())
    }

    pub fn read_i32(&self, offset: usize) -> Result<i32, Trap> {
        if offset + 4 > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        let slice = &self.bytes[offset..offset + 4];
        Ok(i32::from_le_bytes(slice.try_into().unwrap()))
    }

    pub fn write_i32(&mut self, offset: usize, val: i32) -> Result<(), Trap> {
        if offset + 4 > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        self.bytes[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn read_i64(&self, offset: usize) -> Result<i64, Trap> {
        if offset + 8 > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        let slice = &self.bytes[offset..offset + 8];
        Ok(i64::from_le_bytes(slice.try_into().unwrap()))
    }

    pub fn write_i64(&mut self, offset: usize, val: i64) -> Result<(), Trap> {
        if offset + 8 > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        self.bytes[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn memory_fill(&mut self, dest: usize, val: u8, count: usize) -> Result<(), Trap> {
        if dest + count > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        self.bytes[dest..dest + count].fill(val);
        Ok(())
    }

    pub fn memory_copy(&mut self, dest: usize, src: usize, count: usize) -> Result<(), Trap> {
        if dest + count > self.bytes.len() || src + count > self.bytes.len() {
            return Err(Trap::OutOfBoundsMemoryAccess);
        }
        self.bytes.copy_within(src..src + count, dest);
        Ok(())
    }
}
