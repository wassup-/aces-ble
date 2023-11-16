/// Verify the checksum of a payload.
pub fn verify_checksum(checksum: u16, b: &[u8], c: u8) -> bool {
    calculate_checksum(b, c) == checksum
}

pub fn calculate_checksum(b: &[u8], c: u8) -> u16 {
    let x: i32 = c as i32;
    let i2 = b.iter().fold(0i32, |acc, b| acc + *b as i32);
    let val: i32 = (!(i2 + (x + b.len() as i32))) + 1;
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
            calculate_checksum(&[0x0D, 0x0B, 0x0D, 0x0D, 0x0D, 0x0B, 0x0D, 0x0F,], 0x00),
            0xFF92
        );
    }

    use super::*;
}
