pub fn i16_from_bytes(b: &[u8]) -> i16 {
    assert!(b.len() == 2);
    // device uses big endian encoding
    i16::from_be_bytes([b[0], b[1]])
}
