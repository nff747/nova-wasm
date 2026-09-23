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
        let mut result = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_bytes(1)?[0];
            result |= ((byte & 0x7f) as u32) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        Some(result)
    }

    pub fn read_section(&mut self) -> Option<WasmSection<'a>> {
        let id_byte = self.read_bytes(1)?[0];
        let id = SectionId::from_u8(id_byte)?;
        let size = self.read_u32_leb128()?;
        let data = self.read_bytes(size as usize)?;
        Some(WasmSection { id, size, data })
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

#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    Unreachable = 0x00,
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    End = 0x0B,
}
