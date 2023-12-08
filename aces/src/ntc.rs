#[derive(Eq, PartialEq, Debug)]
pub struct NtcList(Vec<i16>);

impl NtcList {
    pub fn from_list(list: Vec<i16>) -> Self {
        NtcList(list)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn avg(&self) -> i16 {
        let init = *self.0.first().unwrap_or(&0);
        let total = self.0.iter().skip(1).fold(init, |acc, ntc| acc + ntc);
        total / self.len() as i16
    }

    pub fn min(&self) -> i16 {
        let init = match self.0.first() {
            Some(first) => *first,
            _ => return 0,
        };
        self.0.iter().skip(1).fold(init, |acc, ntc| acc.min(*ntc))
    }

    pub fn max(&self) -> i16 {
        let init = match self.0.first() {
            Some(first) => *first,
            _ => return 0,
        };
        self.0.iter().skip(1).fold(init, |acc, ntc| acc.max(*ntc))
    }

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
