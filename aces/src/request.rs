#[derive(Debug)]
pub enum Request {
    Clear,
    BatteryDetail,
    BatteryProtect,
    BatteryVoltage,
}

impl Request {
    pub fn is_complete_request(req: &[u8]) -> bool {
        req.len() >= 7
    }

    pub fn parse_request(req: &[u8]) -> ParseResult<Self> {
        if req.len() < 7 {
            return Err(ParseError::NotEnoughData);
        }

        if req == Self::Clear.bytes() {
            return Ok(Self::Clear);
        } else if req == Self::BatteryDetail.bytes() {
            return Ok(Self::BatteryDetail);
        } else if req == Self::BatteryProtect.bytes() {
            return Ok(Self::BatteryProtect);
        } else if req == Self::BatteryVoltage.bytes() {
            return Ok(Self::BatteryVoltage);
        }

        Err(ParseError::InvalidData)
    }

    pub fn bytes(&self) -> &'static [u8] {
        match self {
            Self::Clear => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // "00000000000000"
            Self::BatteryDetail => &[0xdd, 0xa5, 0x03, 0x00, 0xff, 0xfd, 0x77], // "DDA50300FFFD77"
            Self::BatteryProtect => &[0xdd, 0xa5, 0xaa, 0x00, 0xff, 0x56, 0x77], // "DDA5AA00FF5677"
            Self::BatteryVoltage => &[0xdd, 0xa5, 0x04, 0x00, 0xff, 0xfc, 0x77], // "DDA50400FFFC77"
        }
    }
}

use crate::{ParseError, ParseResult};
