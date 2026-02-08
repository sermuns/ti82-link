/// TI-82/83 variable encoding and decoding utilities
use heapless::Vec;

/// Decodes a TI real number (9 bytes) into mantissa and exponent components
/// Returns (mantissa_digits, exponent, is_negative)
pub fn decode_real_raw(bytes: &[u8]) -> Option<([u8; 14], i8, bool)> {
    if bytes.len() != 9 {
        return None;
    }

    let is_negative = bytes[0] & 0x80 != 0;
    let exponent = (bytes[1] as i16 - 0x80) as i8;

    let mut digits = [0u8; 14];
    for i in 0..7 {
        let digit_pair = bytes[2 + i];
        digits[i * 2] = (digit_pair >> 4) & 0x0F;
        digits[i * 2 + 1] = digit_pair & 0x0F;
    }

    Some((digits, exponent, is_negative))
}

/// Encodes a TI real number from raw byte components
pub fn encode_real_raw(bcd_mantissa: &[u8; 7], exponent: i8, is_negative: bool) -> [u8; 9] {
    let mut result = [0u8; 9];

    result[0] = if is_negative { 0x80 } else { 0x00 };
    result[1] = (exponent as i16 + 0x80) as u8;
    result[2..9].copy_from_slice(bcd_mantissa);

    result
}

/// Encode an integer value as a TI real number
pub fn encode_integer(value: i32) -> [u8; 9] {
    let mut result = [0u8; 9];

    if value == 0 {
        result[1] = 0x80;
        return result;
    }

    let is_negative = value < 0;
    let abs_value = if is_negative { -value } else { value } as u32;

    result[0] = if is_negative { 0x80 } else { 0x00 };

    let mut exp: i8 = 0;
    let mut temp = abs_value;
    while temp >= 10 {
        temp /= 10;
        exp += 1;
    }

    result[1] = (exp as i16 + 0x80) as u8;

    let mut bcd = [0u8; 7];
    let mut digits = [0u8; 14];
    let mut temp = abs_value;
    let mut digit_count = 0;

    let mut temp_digits = [0u8; 14];
    for i in 0..14 {
        temp_digits[i] = (temp % 10) as u8;
        temp /= 10;
        if temp == 0 && digit_count == 0 {
            digit_count = i + 1;
        }
    }

    for i in 0..digit_count {
        digits[i] = temp_digits[digit_count - 1 - i];
    }

    for i in 0..7 {
        bcd[i] = (digits[i * 2] << 4) | digits[i * 2 + 1];
    }

    result[2..9].copy_from_slice(&bcd);
    result
}

/// Encodes a list of raw TI real numbers
pub fn encode_list_raw(real_numbers: &[[u8; 9]]) -> Vec<u8, 256> {
    let mut result = Vec::new();

    let size = real_numbers.len() as u16;
    result.push((size & 0xFF) as u8).ok();
    result.push((size >> 8) as u8).ok();

    for real_num in real_numbers {
        for &byte in real_num {
            result.push(byte).ok();
        }
    }

    result
}

/// Decodes a TI list into individual real number bytes
pub fn decode_list_raw(bytes: &[u8]) -> Option<Vec<[u8; 9], 28>> {
    if bytes.len() < 2 {
        return None;
    }

    let size = (bytes[1] as u16) << 8 | (bytes[0] as u16);
    let expected_len = 2 + (size as usize) * 9;

    if bytes.len() != expected_len {
        return None;
    }

    let mut result = Vec::new();
    for i in 0..size {
        let offset = 2 + (i as usize) * 9;
        let mut real_bytes = [0u8; 9];
        real_bytes.copy_from_slice(&bytes[offset..offset + 9]);
        result.push(real_bytes).ok();
    }

    Some(result)
}

/// Encodes a string into TI string format
pub fn encode_string(text: &[u8]) -> Vec<u8, 256> {
    let mut result = Vec::new();

    let len = text.len().min(255) as u16;
    result.push((len & 0xFF) as u8).ok();
    result.push((len >> 8) as u8).ok();

    for &byte in text.iter().take(255) {
        result.push(byte).ok();
    }

    result
}

/// Decodes a TI string into a byte vector
pub fn decode_string(bytes: &[u8]) -> Option<Vec<u8, 256>> {
    if bytes.len() < 2 {
        return None;
    }

    let len = (bytes[1] as u16) << 8 | (bytes[0] as u16);
    let expected_len = 2 + len as usize;

    if bytes.len() != expected_len {
        return None;
    }

    let mut result = Vec::new();
    for i in 2..expected_len {
        result.push(bytes[i]).ok();
    }

    Some(result)
}
