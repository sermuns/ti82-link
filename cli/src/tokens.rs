// TI-82 Token decoder
// Based on TI-82 Link Protocol Guide token table

use std::collections::HashMap;

pub struct TokenDecoder {
    tokens: HashMap<u8, String>,
    two_byte_tokens: HashMap<(u8, u8), String>,
}

impl TokenDecoder {
    pub fn new() -> Self {
        let mut tokens = HashMap::new();
        let mut two_byte_tokens = HashMap::new();

        // Digits and basic operators
        tokens.insert(0x30, "0".to_string());
        tokens.insert(0x31, "1".to_string());
        tokens.insert(0x32, "2".to_string());
        tokens.insert(0x33, "3".to_string());
        tokens.insert(0x34, "4".to_string());
        tokens.insert(0x35, "5".to_string());
        tokens.insert(0x36, "6".to_string());
        tokens.insert(0x37, "7".to_string());
        tokens.insert(0x38, "8".to_string());
        tokens.insert(0x39, "9".to_string());
        tokens.insert(0x2B, "+".to_string());
        tokens.insert(0x2D, "-".to_string());
        tokens.insert(0x2A, "*".to_string());
        tokens.insert(0x2F, "/".to_string());
        tokens.insert(0x28, "(".to_string());
        tokens.insert(0x29, ")".to_string());
        tokens.insert(0x2C, ",".to_string());
        tokens.insert(0x3A, ":".to_string());
        tokens.insert(0x3D, "=".to_string());
        tokens.insert(0x20, " ".to_string());
        tokens.insert(0x22, "\"".to_string());
        tokens.insert(0x2E, ".".to_string());

        // Variable names
        for i in b'A'..=b'Z' {
            tokens.insert(i, (i as char).to_string());
        }

        // Special characters
        tokens.insert(0x04, "→".to_string()); // Store arrow
        tokens.insert(0x0D, "\n".to_string()); // Newline
        tokens.insert(0x3F, "?".to_string()); // Question mark

        // Common commands (0xB0-0xBF range)
        tokens.insert(0xB0, "sin(".to_string());
        tokens.insert(0xB1, "cos(".to_string());
        tokens.insert(0xB2, "tan(".to_string());
        tokens.insert(0xB3, "sin⁻¹(".to_string());
        tokens.insert(0xB4, "cos⁻¹(".to_string());
        tokens.insert(0xB5, "tan⁻¹(".to_string());
        tokens.insert(0xB6, "√(".to_string());
        tokens.insert(0xB7, "²".to_string());
        tokens.insert(0xB8, "^".to_string());
        tokens.insert(0xB9, "log(".to_string());
        tokens.insert(0xBA, "ln(".to_string());
        tokens.insert(0xBB, "e^(".to_string());
        tokens.insert(0xBC, "abs(".to_string());
        tokens.insert(0xBD, "int(".to_string());
        tokens.insert(0xBE, "round(".to_string());
        tokens.insert(0xBF, "iPart(".to_string());

        // Control flow (0xD0-0xDF range)
        tokens.insert(0xD0, "If ".to_string());
        tokens.insert(0xD1, "Then".to_string());
        tokens.insert(0xD2, "Else".to_string());
        tokens.insert(0xD3, "While ".to_string());
        tokens.insert(0xD4, "Repeat ".to_string());
        tokens.insert(0xD5, "For(".to_string());
        tokens.insert(0xD6, "End".to_string());
        tokens.insert(0xD7, "Return".to_string());
        tokens.insert(0xD8, "Lbl ".to_string());
        tokens.insert(0xD9, "Goto ".to_string());
        tokens.insert(0xDA, "Pause".to_string());
        tokens.insert(0xDB, "Stop".to_string());
        tokens.insert(0xDC, "IS>(".to_string());
        tokens.insert(0xDD, "DS<(".to_string());
        tokens.insert(0xDE, "Input ".to_string());
        tokens.insert(0xDF, "Prompt ".to_string());

        // I/O commands (0xE0-0xEF range)
        tokens.insert(0xE0, "Disp ".to_string());
        tokens.insert(0xE1, "DispGraph".to_string());
        tokens.insert(0xE2, "Output(".to_string());
        tokens.insert(0xE3, "ClrHome".to_string());
        tokens.insert(0xE4, "Fill(".to_string());
        tokens.insert(0xE5, "SortA(".to_string());
        tokens.insert(0xE6, "SortD(".to_string());
        tokens.insert(0xE7, "DispTable".to_string());
        tokens.insert(0xE8, "Menu(".to_string());
        tokens.insert(0xE9, "Send(".to_string());
        tokens.insert(0xEA, "Get(".to_string());

        // Comparison operators
        tokens.insert(0x6A, "and ".to_string());
        tokens.insert(0x6B, "or ".to_string());
        tokens.insert(0x6C, "xor ".to_string());
        tokens.insert(0x6D, "not(".to_string());
        tokens.insert(0x10, "√".to_string());
        tokens.insert(0x11, "⁻¹".to_string());
        tokens.insert(0x23, "#".to_string());
        tokens.insert(0x72, "r".to_string());
        tokens.insert(0x58, "X".to_string());

        // Two-byte tokens (prefix, suffix)
        two_byte_tokens.insert((0x7E, 0x00), "ClrList ".to_string());
        two_byte_tokens.insert((0x7E, 0x01), "ClrTable".to_string());
        two_byte_tokens.insert((0x7E, 0x02), "Histogram".to_string());
        two_byte_tokens.insert((0x7E, 0x03), "xyLine".to_string());
        two_byte_tokens.insert((0x7E, 0x04), "Scatter".to_string());
        two_byte_tokens.insert((0x7E, 0x05), "LinReg".to_string());

        Self {
            tokens,
            two_byte_tokens,
        }
    }

    pub fn decode(&self, data: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;

        while i < data.len() {
            let byte = data[i];

            // Check for two-byte tokens
            if i + 1 < data.len() {
                let next_byte = data[i + 1];
                if let Some(token) = self.two_byte_tokens.get(&(byte, next_byte)) {
                    result.push_str(token);
                    i += 2;
                    continue;
                }
            }

            // Single-byte token
            if let Some(token) = self.tokens.get(&byte) {
                result.push_str(token);
            } else {
                // Unknown token - show as hex
                result.push_str(&format!("[{byte:02X}]"));
            }

            i += 1;
        }

        result
    }

    pub fn decode_program(&self, data: &[u8]) -> Option<String> {
        if data.len() < 2 {
            return None;
        }

        // First 2 bytes are the length (little-endian)
        let length = u16::from_le_bytes([data[0], data[1]]) as usize;

        if data.len() < 2 + length {
            return None;
        }

        // Decode the token data (skip the 2-byte length header)
        Some(self.decode(&data[2..2 + length]))
    }
}

impl Default for TokenDecoder {
    fn default() -> Self {
        Self::new()
    }
}
