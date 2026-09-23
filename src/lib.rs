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
