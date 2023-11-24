#[derive(Eq, PartialEq, Debug)]
pub struct BatteryVoltage(i16, i16, i16, i16);

impl BatteryVoltage {
    /// The total voltage.
    pub fn total(&self) -> i64 {
        self.0 as i64 + self.1 as i64 + self.2 as i64 + self.3 as i64
    }

    /// The average voltage.
    pub fn avg(&self) -> i64 {
        self.total() / 4
    }

    /// The minimum voltage.
    pub fn min(&self) -> i64 {
        self.0.min(self.1).min(self.2).min(self.3) as i64
    }

    /// The maximum voltage.
    pub fn max(&self) -> i64 {
        self.0.max(self.1).max(self.2).max(self.3) as i64
    }

    pub fn parse_message(msg: &[u8]) -> ParseResult<BatteryVoltage> {
        if msg.len() < 8 {
            return Err(ParseError::NotEnoughData);
        }

        Ok(BatteryVoltage(
            i16_from_bytes(&msg[0..2]),
            i16_from_bytes(&msg[2..4]),
            i16_from_bytes(&msg[4..6]),
            i16_from_bytes(&msg[6..8]),
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_message() {
        assert_eq!(
            BatteryVoltage::parse_message(&[1, 2, 3, 4, 5, 6, 7]),
            Err(ParseError::NotEnoughData)
        );

        assert_eq!(
            BatteryVoltage::parse_message(&[0x0d, 0x0b, 0x0d, 0x0d, 0x0d, 0x0b, 0x0d, 0x0f]),
            Ok(BatteryVoltage(3339, 3341, 3339, 3343))
        );
    }

    use super::*;
}
use super::{util::i16_from_bytes, ParseError, ParseResult};
