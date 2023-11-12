pub enum Message {
    BatteryVoltage(BatteryVoltage),
    Detail(BatteryDetail),
    Protect(BatteryProtect),
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse message")]
pub struct ParseMessageFailed;

impl Message {
    pub fn parse_message(msg: &[u8]) -> ParseResult<Message> {
        if let Some(msg) = msg.strip_prefix(&[0xdd, 0x03]) {
            return BatteryDetail::parse_message(msg).map(Message::Detail);
        } else if let Some(msg) = msg.strip_prefix(&[0xdd, 0x04]) {
            return BatteryVoltage::parse_message(msg).map(Message::BatteryVoltage);
        } else if let Some(msg) = msg.strip_prefix(&[0xdd, 0xaa]) {
            return BatteryProtect::parse_message(msg).map(Message::Protect);
        }

        Err(ParseError::InvalidData)
    }
}

use super::{BatteryDetail, BatteryProtect, BatteryVoltage, ParseError, ParseResult};
