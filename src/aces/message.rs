#[derive(Eq, PartialEq, Debug)]
pub enum Message {
    Voltage(BatteryVoltage),
    Detail(BatteryDetail),
    Protect(BatteryProtect),
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse message")]
pub struct ParseMessageFailed;

impl Message {
    pub fn parse_message(msg: &[u8]) -> ParseResult<Message> {
        if msg.len() < 7 {
            return Err(ParseError::NotEnoughData);
        }

        let identifier = u16_from_bytes(&msg[..2]);
        let control = msg[2];
        let len = msg[3] as usize;
        let checksum = u16_from_bytes(&msg[msg.len() - 3..msg.len() - 1]);

        if msg.len() != 7 + len {
            return Err(ParseError::NotEnoughData);
        }

        let payload = &msg[4..(4 + len)];
        if !verify_checksum(checksum, payload, control) {
            return Err(ParseError::InvalidChecksum);
        }

        match identifier {
            0xdd03 => return BatteryDetail::parse_message(payload).map(Message::Detail),
            0xdd04 => return BatteryVoltage::parse_message(payload).map(Message::Voltage),
            0xddaa => return BatteryProtect::parse_message(payload).map(Message::Protect),
            _ => (),
        }

        Err(ParseError::InvalidData)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_message() {
        assert_eq!(
            Message::parse_message(&[
                0xDD, 0x04, 0x00, 0x08, 0x0D, 0xE2, 0x0D, 0xDC, 0x0D, 0xEC, 0x0D, 0xED, 0xFF, 0x00,
                0x77
            ]),
            Err(ParseError::InvalidChecksum)
        );
        assert!(Message::parse_message(&[
            0xDD, 0x04, 0x00, 0x08, 0x0D, 0xE2, 0x0D, 0xDC, 0x0D, 0xEC, 0x0D, 0xED, 0xFC, 0x2D,
            0x77
        ])
        .is_ok());
        assert_eq!(
            Message::parse_message(&[
                0xDD, 0x03, 0x00, 0x1D, 0x05, 0x38, 0x02, 0x83, 0x17, 0x5C, 0x27, 0xDE, 0x00, 0x09,
                0x2B, 0x94, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x3B, 0x03, 0x04, 0x03, 0x0B,
                0x7F, 0x0B, 0x6C, 0x0B, 0x69, 0xFB, 0x07, 0x77
            ]),
            Ok(Message::Detail(BatteryDetail {
                total_voltage: 1336,
                current: 643,
                residual_capacity: 5980,
                standard_capacity: 10206,
                cycles: 9,
                date_of_production: 11156,
                equilibrium: 0,
                equilibrium_high: 0,
                protection_of_state: 0,
                software_version: 32,
                residual_capacity_percent: 59,
                control_state: 3,
                charge: true,
                discharge: true,
                battery_number: 4,
                list_ntc: NtcList::from_list(vec![212, 193, 190])
            }))
        );
    }

    use crate::aces::NtcList;

    use super::*;
}

use super::{
    util::u16_from_bytes, verify_checksum, BatteryDetail, BatteryProtect, BatteryVoltage,
    ParseError, ParseResult,
};
