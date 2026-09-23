pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d]; // \0asm
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

pub struct WasmModule {
    pub magic: [u8; 4],
    pub version: [u8; 4],
}

pub struct Decoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        if self.offset + len > self.data.len() {
            None
        } else {
            let res = &self.data[self.offset..self.offset + len];
            self.offset += len;
            Some(res)
        }
    }
}
