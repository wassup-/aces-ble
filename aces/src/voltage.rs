#[derive(Eq, PartialEq, Debug)]
pub struct BatteryVoltage(pub Vec<i16>);

impl BatteryVoltage {
    pub fn parse_message(msg: &[u8]) -> ParseResult<BatteryVoltage> {
        if msg.is_empty() {
            return Err(ParseError::NotEnoughData);
        }
        if msg.len() % 2 != 0 {
            return Err(ParseError::NotEnoughData);
        }

        let num_items = msg.len() / 2;
        let mut list = Vec::new();

        for i in 0..num_items {
            let offset = i * 2;
            list.push(i16_from_bytes(&msg[offset..(offset + 2)]));
        }

        Ok(BatteryVoltage(list))
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
            BatteryVoltage::parse_message(&[0x0d, 0x0b, 0x0d, 0x0d, 0x0d, 0x0f]),
            Ok(BatteryVoltage(vec![3339, 3341, 3343]))
        );

        assert_eq!(
            BatteryVoltage::parse_message(&[0x0d, 0x0b, 0x0d, 0x0d, 0x0d, 0x0b, 0x0d, 0x0f]),
            Ok(BatteryVoltage(vec![3339, 3341, 3339, 3343]))
        );
    }

    use super::*;
}
use super::{util::i16_from_bytes, ParseError, ParseResult};
