//! LEB128 (Little Endian Base 128) variable-length integer encoders.

/// Encodes an unsigned 32-bit integer into LEB128 format.
pub fn encode_u32(mut val: u32, buf: &mut Vec<u8>) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            buf.push(byte | 0x80);
        } else {
            buf.push(byte);
            break;
        }
    }
}

/// Encodes a signed 32-bit integer into LEB128 format.
pub fn encode_i32(mut val: i32, buf: &mut Vec<u8>) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        let sign_bit = (byte & 0x40) != 0;
        if (val == 0 && !sign_bit) || (val == -1 && sign_bit) {
            buf.push(byte);
            break;
        } else {
            buf.push(byte | 0x80);
        }
    }
}

/// Encodes a signed 64-bit integer into LEB128 format.
pub fn encode_i64(mut val: i64, buf: &mut Vec<u8>) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        let sign_bit = (byte & 0x40) != 0;
        if (val == 0 && !sign_bit) || (val == -1 && sign_bit) {
            buf.push(byte);
            break;
        } else {
            buf.push(byte | 0x80);
        }
    }
}

/// Encodes a UTF-8 string prefixed with its LEB128 byte length.
pub fn encode_name(name: &str, buf: &mut Vec<u8>) {
    encode_u32(name.len() as u32, buf);
    buf.extend_from_slice(name.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_u32() {
        let mut buf = Vec::new();
        encode_u32(624485, &mut buf);
        assert_eq!(buf, vec![0xE5, 0x8E, 0x26]);

        buf.clear();
        encode_u32(0, &mut buf);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        encode_u32(1, &mut buf);
        assert_eq!(buf, vec![0x01]);
    }

    #[test]
    fn test_encode_i32() {
        let mut buf = Vec::new();
        encode_i32(-624485, &mut buf);
        assert_eq!(buf, vec![0x9B, 0xF1, 0x59]);

        buf.clear();
        encode_i32(-1, &mut buf);
        assert_eq!(buf, vec![0x7F]);
    }
}
