//! Nova-WASM: Zero-copy, high-throughput WebAssembly binary format decoder and AST validator.

pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d]; // \0asm
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValType {
    I32 = 0x7F,
    I64 = 0x7E,
    F32 = 0x7D,
    F64 = 0x7C,
    V128 = 0x7B,
    FuncRef = 0x70,
    ExternRef = 0x6F,
}

impl ValType {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x7F => Some(ValType::I32),
            0x7E => Some(ValType::I64),
            0x7D => Some(ValType::F32),
            0x7C => Some(ValType::F64),
            0x7B => Some(ValType::V128),
            0x70 => Some(ValType::FuncRef),
            0x6F => Some(ValType::ExternRef),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportEntry {
    pub name: String,
    pub kind: ExportKind,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Func = 0x00,
    Table = 0x01,
    Memory = 0x02,
    Global = 0x03,
}

impl ExportKind {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(ExportKind::Func),
            0x01 => Some(ExportKind::Table),
            0x02 => Some(ExportKind::Memory),
            0x03 => Some(ExportKind::Global),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    Unreachable,
    Nop,
    Block,
    Loop,
    If,
    Else,
    End,
    Br(u32),
    BrIf(u32),
    Return,
    Call(u32),
    Drop,
    Select,
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    I32Const(i32),
    I64Const(i64),
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    Unknown(u8),
}

pub struct Decoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_magic(&mut self) -> Option<[u8; 4]> {
        let bytes = self.read_bytes(4)?;
        let mut magic = [0u8; 4];
        magic.copy_from_slice(bytes);
        if magic == WASM_MAGIC {
            Some(magic)
        } else {
            None
        }
    }

    pub fn read_version(&mut self) -> Option<[u8; 4]> {
        let bytes = self.read_bytes(4)?;
        let mut version = [0u8; 4];
        version.copy_from_slice(bytes);
        if version == WASM_VERSION {
            Some(version)
        } else {
            None
        }
    }

    pub fn read_u32_leb128(&mut self) -> Option<u32> {
        let mut result = 0u32;
        let mut shift = 0;
        loop {
            let byte = self.read_bytes(1)?[0];
            result |= ((byte & 0x7f) as u32) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
            if shift >= 35 {
                return None; // Overflow guard
            }
        }
        Some(result)
    }

    pub fn read_i32_leb128(&mut self) -> Option<i32> {
        let mut result = 0i32;
        let mut shift = 0;
        let mut byte: u8;
        loop {
            byte = self.read_bytes(1)?[0];
            result |= ((byte & 0x7f) as i32) << shift;
            shift += 7;
            if (byte & 0x80) == 0 {
                break;
            }
            if shift >= 35 {
                return None;
            }
        }
        if shift < 32 && (byte & 0x40) != 0 {
            result |= !0 << shift;
        }
        Some(result)
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

    pub fn read_section(&mut self) -> Option<WasmSection<'a>> {
        let id_byte = self.read_bytes(1)?[0];
        let id = SectionId::from_u8(id_byte)?;
        let size = self.read_u32_leb128()?;
        let data = self.read_bytes(size as usize)?;
        Some(WasmSection { id, size, data })
    }

    pub fn decode_export_section(&mut self) -> Option<Vec<ExportEntry>> {
        let count = self.read_u32_leb128()?;
        let mut exports = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let name_len = self.read_u32_leb128()? as usize;
            let name_bytes = self.read_bytes(name_len)?;
            let name = std::str::from_utf8(name_bytes).ok()?.to_string();
            let kind_byte = self.read_bytes(1)?[0];
            let kind = ExportKind::from_u8(kind_byte)?;
            let index = self.read_u32_leb128()?;
            exports.push(ExportEntry { name, kind, index });
        }

        Some(exports)
    }

    pub fn decode_next_opcode(&mut self) -> Option<Opcode> {
        let byte = self.read_bytes(1)?[0];
        let op = match byte {
            0x00 => Opcode::Unreachable,
            0x01 => Opcode::Nop,
            0x02 => Opcode::Block,
            0x03 => Opcode::Loop,
            0x04 => Opcode::If,
            0x05 => Opcode::Else,
            0x0B => Opcode::End,
            0x0C => Opcode::Br(self.read_u32_leb128()?),
            0x0D => Opcode::BrIf(self.read_u32_leb128()?),
            0x0F => Opcode::Return,
            0x10 => Opcode::Call(self.read_u32_leb128()?),
            0x1A => Opcode::Drop,
            0x1B => Opcode::Select,
            0x20 => Opcode::LocalGet(self.read_u32_leb128()?),
            0x21 => Opcode::LocalSet(self.read_u32_leb128()?),
            0x22 => Opcode::LocalTee(self.read_u32_leb128()?),
            0x23 => Opcode::GlobalGet(self.read_u32_leb128()?),
            0x24 => Opcode::GlobalSet(self.read_u32_leb128()?),
            0x41 => Opcode::I32Const(self.read_i32_leb128()?),
            0x6A => Opcode::I32Add,
            0x6B => Opcode::I32Sub,
            0x6C => Opcode::I32Mul,
            0x6D => Opcode::I32DivS,
            0x6E => Opcode::I32DivU,
            other => Opcode::Unknown(other),
        };
        Some(op)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SectionId {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    DataCount = 12,
}

impl SectionId {
    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            0 => Some(SectionId::Custom),
            1 => Some(SectionId::Type),
            2 => Some(SectionId::Import),
            3 => Some(SectionId::Function),
            4 => Some(SectionId::Table),
            5 => Some(SectionId::Memory),
            6 => Some(SectionId::Global),
            7 => Some(SectionId::Export),
            8 => Some(SectionId::Start),
            9 => Some(SectionId::Element),
            10 => Some(SectionId::Code),
            11 => Some(SectionId::Data),
            12 => Some(SectionId::DataCount),
            _ => None,
        }
    }
}

pub struct WasmSection<'a> {
    pub id: SectionId,
    pub size: u32,
    pub data: &'a [u8],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_and_version() {
        let mut binary = Vec::new();
        binary.extend_from_slice(&WASM_MAGIC);
        binary.extend_from_slice(&WASM_VERSION);

        let mut decoder = Decoder::new(&binary);
        assert_eq!(decoder.read_magic(), Some(WASM_MAGIC));
        assert_eq!(decoder.read_version(), Some(WASM_VERSION));
    }

    #[test]
    fn test_leb128_signed_and_unsigned() {
        let unsigned = [0xE5, 0x8E, 0x26];
        let mut d1 = Decoder::new(&unsigned);
        assert_eq!(d1.read_u32_leb128(), Some(624485));

        // -624485 in signed LEB128: 0x9B 0xF1 0x59
        let signed = [0x9B, 0xF1, 0x59];
        let mut d2 = Decoder::new(&signed);
        assert_eq!(d2.read_i32_leb128(), Some(-624485));
    }

    #[test]
    fn test_opcode_decoding() {
        // i32.const 42, local.get 0, i32.add, end
        let code = [0x41, 0x2A, 0x20, 0x00, 0x6A, 0x0B];
        let mut decoder = Decoder::new(&code);

        assert_eq!(decoder.decode_next_opcode(), Some(Opcode::I32Const(42)));
        assert_eq!(decoder.decode_next_opcode(), Some(Opcode::LocalGet(0)));
        assert_eq!(decoder.decode_next_opcode(), Some(Opcode::I32Add));
        assert_eq!(decoder.decode_next_opcode(), Some(Opcode::End));
    }

    #[test]
    fn test_export_decoding() {
        // 1 export: "add", func index 0
        let export_data = [
            0x01, // 1 export
            0x03, b'a', b'd', b'd', // name "add"
            0x00, // ExportKind::Func
            0x00, // index 0
        ];
        let mut decoder = Decoder::new(&export_data);
        let exports = decoder.decode_export_section().unwrap();
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "add");
        assert_eq!(exports[0].kind, ExportKind::Func);
        assert_eq!(exports[0].index, 0);
    }
}
