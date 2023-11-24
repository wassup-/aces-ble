/// # Panics
///
/// This function will panic if `b.len() != 2`.
pub fn i16_from_bytes(b: &[u8]) -> i16 {
    assert!(b.len() == 2);
    // device uses big endian encoding
    i16::from_be_bytes([b[0], b[1]])
}

/// # Panics
///
/// This function will panic if `b.len() != 2`.
pub fn u16_from_bytes(b: &[u8]) -> u16 {
    assert!(b.len() == 2);
    // device uses big endian encoding
    u16::from_be_bytes([b[0], b[1]])
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_i16_from_bytes() {
        assert_eq!(i16_from_bytes(&[0x00, 0x00]), 0x0000);
        assert_eq!(i16_from_bytes(&[0x12, 0x34]), 0x1234);
        assert_eq!(i16_from_bytes(&[0x56, 0x23]), 0x5623);
    }

    use super::*;
}
