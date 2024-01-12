#[derive(Eq, PartialEq, Debug)]
pub struct NtcList(pub Vec<i16>);

impl NtcList {
    pub fn parse_message(msg: &[u8]) -> ParseResult<Self> {
        if msg.is_empty() {
            return Err(ParseError::NotEnoughData);
        }

        let num_ntc = msg[0] as usize;
        if msg.len() < 1 + (num_ntc * 2) {
            return Err(ParseError::NotEnoughData);
        }

        let mut list = Vec::new();

        for i in 0..num_ntc {
            let offset = 1 + (i * 2);
            list.push(i16_from_bytes(&msg[offset..(offset + 2)]) - 2731);
        }

        Ok(NtcList(list))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_message() {
        assert_eq!(NtcList::parse_message(&[0]), Ok(NtcList(Vec::new())));
        assert_eq!(
            NtcList::parse_message(&[1, 1]),
            Err(ParseError::NotEnoughData)
        );
        assert_eq!(
            NtcList::parse_message(&[1, 0x0b, 0xad]),
            Ok(NtcList(vec![0x0102]))
        );
        assert_eq!(
            NtcList::parse_message(&[2, 0x0b, 0xad, 0x0d]),
            Err(ParseError::NotEnoughData)
        );
        assert_eq!(
            NtcList::parse_message(&[2, 0x0b, 0xad, 0x0d, 0xaf]),
            Ok(NtcList(vec![0x0102, 0x0304]))
        );
    }

    use super::*;
}

use super::{util::i16_from_bytes, ParseError, ParseResult};
