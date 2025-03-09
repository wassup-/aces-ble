#[derive(Eq, PartialEq, Debug)]
pub enum Response {
    BatteryDetail(BatteryDetail),
    BatteryProtect(BatteryProtect),
    BatteryVoltage(BatteryVoltage),
}

impl Response {
    pub fn is_complete_response(response: &[u8]) -> bool {
        if response.len() < 7 {
            return false;
        }

        let len = response[3] as usize;
        response.len() >= 7 + len
    }

    pub fn parse_response(response: &[u8]) -> ParseResult<Self> {
        if response.len() < 7 {
            return Err(ParseError::NotEnoughData);
        }

        let identifier = u16_from_bytes(&response[..2]);
        let control = response[2];
        let len = response[3] as usize;
        let checksum = u16_from_bytes(&response[response.len() - 3..response.len() - 1]);

        if response.len() != 7 + len {
            return Err(ParseError::NotEnoughData);
        }

        let payload = &response[4..(4 + len)];
        if !verify_checksum(checksum, payload, control) {
            return Err(ParseError::InvalidChecksum);
        }

        match identifier {
            0xdd03 => return BatteryDetail::parse_message(payload).map(Response::BatteryDetail),
            0xdd04 => return BatteryVoltage::parse_message(payload).map(Response::BatteryVoltage),
            0xddaa => return BatteryProtect::parse_message(payload).map(Response::BatteryProtect),
            _ => (),
        }

        Err(ParseError::InvalidData)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_complete_response() {
        assert!(!Response::is_complete_response(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]));
        assert!(!Response::is_complete_response(&[
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00
        ]));
        assert!(Response::is_complete_response(&[
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00
        ]));
    }

    #[test]
    fn test_parse_message() {
        assert_eq!(
            Response::parse_response(&[
                0xdd, 0x04, 0x00, 0x08, 0x0d, 0xe2, 0x0d, 0xdc, 0x0d, 0xec, 0x0d, 0xed, 0xca, 0xfe,
                0x77
            ]),
            Err(ParseError::InvalidChecksum)
        );

        assert!(Response::parse_response(&[
            0xdd, 0x04, 0x00, 0x08, 0x0d, 0xe2, 0x0d, 0xdc, 0x0d, 0xec, 0x0d, 0xed, 0xfc, 0x2d,
            0x77
        ])
        .is_ok());

        assert_eq!(
            Response::parse_response(&[
                0xdd, 0x03, 0x00, 0x1d, 0x05, 0x38, 0x02, 0x83, 0x17, 0x5c, 0x27, 0xde, 0x00, 0x09,
                0x2b, 0x94, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x3b, 0x03, 0x04, 0x03, 0x0b,
                0x7f, 0x0b, 0x6c, 0x0b, 0x69, 0xfb, 0x07, 0x77
            ]),
            Ok(Response::BatteryDetail(BatteryDetail {
                total_voltage: 1336,
                current: 643,
                residual_capacity: 5980,
                standard_capacity: 10206,
                cycles: 9,
                date_of_production: 11156,
                equilibrium: 0,
                equilibrium_high: 0,
                protection_of_state: ProtectionOfState(0),
                software_version: 32,
                residual_capacity_percent: 59,
                control_state: 3,
                charge: true,
                discharge: true,
                battery_number: 4,
                list_ntc: vec![212, 193, 190]
            }))
        );

        assert_eq!(
            Response::parse_response(&[
                0xdd, 0xaa, 0x00, 0x16, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xe6,
                0x77
            ]),
            Ok(Response::BatteryProtect(BatteryProtect {
                short_circuit: 0,
                over_current_charging: 0,
                over_current_discharging: 0,
                cell_overvoltage: 0,
                cell_undervoltage: 4,
                high_temp_charging: 0,
                low_temp_charging: 0,
                high_temp_discharging: 0,
                low_temp_discharging: 0,
                pack_overvoltage: 0,
                pack_undervoltage: 0
            }))
        )
    }

    use super::*;
    use crate::ProtectionOfState;
}

use crate::{
    util::u16_from_bytes, verify_checksum, BatteryDetail, BatteryProtect, BatteryVoltage,
    ParseError, ParseResult,
};
