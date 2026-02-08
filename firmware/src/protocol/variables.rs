use heapless::Vec;

pub const TYPE_REAL: u8 = 0x00;
pub const TYPE_LIST: u8 = 0x01;
pub const TYPE_MATRIX: u8 = 0x02;
pub const TYPE_EQUATION: u8 = 0x03;
pub const TYPE_STRING: u8 = 0x04;
pub const TYPE_PROGRAM: u8 = 0x05;
pub const TYPE_PROGRAM_LOCKED: u8 = 0x06;
pub const TYPE_PICTURE: u8 = 0x07;
pub const TYPE_GDB: u8 = 0x08;
pub const TYPE_WINDOW: u8 = 0x0B;
pub const TYPE_BACKUP: u8 = 0x0F;

pub struct VariableHeader {
    pub data_size: u16,
    pub type_id: u8,
    pub name: [u8; 8],
}

impl VariableHeader {
    pub fn new(type_id: u8, name: &[u8], data_size: u16) -> Self {
        let mut name_array = [0u8; 8];
        let len = name.len().min(8);
        name_array[..len].copy_from_slice(&name[..len]);
        Self {
            data_size,
            type_id,
            name: name_array,
        }
    }

    pub fn new_real(name_char: u8, data_size: u16) -> Self {
        let mut name = [0u8; 8];
        name[0] = name_char;
        Self {
            data_size,
            type_id: TYPE_REAL,
            name,
        }
    }

    pub fn new_list(list_num: u8, data_size: u16) -> Self {
        let mut name = [0u8; 8];
        // List names: L1-L6 are stored as 0x01-0x06
        name[0] = list_num;
        Self {
            data_size,
            type_id: TYPE_LIST,
            name,
        }
    }

    pub fn new_string(string_num: u8, data_size: u16) -> Self {
        let mut name = [0u8; 8];
        // String names: Str0-Str9 are stored as 0xAA + digit
        name[0] = 0xAA;
        name[1] = string_num;
        Self {
            data_size,
            type_id: TYPE_STRING,
            name,
        }
    }

    pub fn new_program(name: &[u8], data_size: u16) -> Self {
        let mut name_array = [0u8; 8];
        let len = name.len().min(8);
        name_array[..len].copy_from_slice(&name[..len]);
        Self {
            data_size,
            type_id: TYPE_PROGRAM,
            name: name_array,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8, 11> {
        let mut bytes = Vec::new();

        bytes.push((self.data_size & 0xFF) as u8).ok();
        bytes.push((self.data_size >> 8) as u8).ok();
        bytes.push(self.type_id).ok();

        for &byte in self.name.iter() {
            bytes.push(byte).ok();
        }

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 11 {
            return None;
        }

        let data_size = (bytes[1] as u16) << 8 | (bytes[0] as u16);
        let type_id = bytes[2];

        let mut name = [0u8; 8];
        name.copy_from_slice(&bytes[3..11]);

        Some(Self {
            data_size,
            type_id,
            name,
        })
    }

    /// Get human-readable name as a string (for debugging)
    pub fn name_str(&self) -> &[u8] {
        // Find first null byte
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(8);
        &self.name[..len]
    }
}
