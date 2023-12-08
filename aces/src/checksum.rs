/// Verify the checksum of a payload.
///
/// # Parameters
/// ---
/// * `checksum` - The checksum to verify.
/// * `payload` - The payload to verify the checksum for.
/// * `control` - The control byte.
pub fn verify_checksum(checksum: u16, payload: &[u8], control: u8) -> bool {
    calculate_checksum(payload, control) == checksum
}

/// Calculate the checksum of a payload.
///
/// # Parameters
/// ---
/// * `payload` - The payload bytes.
/// * `control` - The control byte.
pub fn calculate_checksum(payload: &[u8], control: u8) -> u16 {
    let x: i32 = control as i32;
    let i2 = payload.iter().fold(0i32, |acc, b| acc + *b as i32);
    let val: i32 = (!(i2 + (x + payload.len() as i32))) + 1;
    (val & 0xffff) as u16
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_checksum() {
        assert!(verify_checksum(0xfffd, &[], 0x03));
        assert!(verify_checksum(0xfffc, &[], 0x04));
        assert!(verify_checksum(0xff56, &[], 0xaa));
    }

    #[test]
    fn test_calculate_checksum() {
        assert_eq!(calculate_checksum(&[], 0x03), 0xfffd);
        assert_eq!(calculate_checksum(&[], 0x04), 0xfffc);
        assert_eq!(calculate_checksum(&[], 0xaa), 0xff56);
        assert_eq!(
            calculate_checksum(&[0x0d, 0x0b, 0x0d, 0x0d, 0x0d, 0x0b, 0x0d, 0x0f,], 0x00),
            0xFF92
        );
    }

    use super::*;
}
