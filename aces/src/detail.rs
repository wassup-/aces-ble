#[derive(Eq, PartialEq, Debug)]
pub struct BatteryDetail {
    /// Total voltage (V).
    pub total_voltage: i16,
    /// Total current (A).
    pub current: i16,
    /// Residual capacity (Ah).
    pub residual_capacity: i16,
    /// Standard capacity (Ah).
    pub standard_capacity: i16,
    /// Cycles.
    pub cycles: i16,
    pub date_of_production: i16,
    pub equilibrium: i16,
    pub equilibrium_high: i16,
    // pub balance_states:Vec<Bool>,
    pub protection_of_state: i16,
    pub software_version: u8,
    pub residual_capacity_percent: u8,
    pub control_state: u8,
    pub charge: bool,
    pub discharge: bool,
    pub battery_number: u8,
    pub list_ntc: NtcList,
}

impl BatteryDetail {
    pub fn parse_message(msg: &[u8]) -> ParseResult<BatteryDetail> {
        if msg.len() < 22 {
            return Err(ParseError::NotEnoughData);
        }

        Ok(BatteryDetail {
            total_voltage: i16_from_bytes(&msg[0..2]),
            current: i16_from_bytes(&msg[2..4]),
            residual_capacity: i16_from_bytes(&msg[4..6]),
            standard_capacity: i16_from_bytes(&msg[6..8]),
            cycles: i16_from_bytes(&msg[8..10]),
            date_of_production: i16_from_bytes(&msg[10..12]),
            equilibrium: i16_from_bytes(&msg[12..14]),
            equilibrium_high: i16_from_bytes(&msg[14..16]),
            protection_of_state: i16_from_bytes(&msg[16..18]),
            software_version: msg[18],
            residual_capacity_percent: msg[19],
            control_state: msg[20],
            charge: (msg[20] & 1) == 1,
            discharge: (msg[20] & 2) == 2,
            battery_number: msg[21],
            list_ntc: NtcList::parse_message(&msg[22..])?,
        })
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_message() {
        assert_eq!(
            BatteryDetail::parse_message(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21
            ]),
            Err(ParseError::NotEnoughData)
        );

        assert_eq!(
            BatteryDetail::parse_message(&[
                0x05, 0x35, 0x00, 0x00, 0x24, 0xB7, 0x27, 0xDE, 0x00, 0x0A, 0x2B, 0x94, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x20, 0x5C, 0x03, 0x04, 0x03, 0x0B, 0x84, 0x0B, 0x79, 0x0B,
            ]),
            Err(ParseError::NotEnoughData)
        );

        assert_eq!(
            BatteryDetail::parse_message(&[
                0x05, 0x35, 0x00, 0x00, 0x24, 0xB7, 0x27, 0xDE, 0x00, 0x0A, 0x2B, 0x94, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x20, 0x5C, 0x03, 0x04, 0x03, 0x0B, 0x84, 0x0B, 0x79, 0x0B,
                0x75,
            ]),
            Ok(BatteryDetail {
                total_voltage: 1333,
                current: 0,
                residual_capacity: 9399,
                standard_capacity: 10206,
                cycles: 10,
                date_of_production: 11156,
                equilibrium: 0,
                equilibrium_high: 0,
                protection_of_state: 0,
                software_version: 32,
                residual_capacity_percent: 92,
                control_state: 3,
                charge: true,
                discharge: true,
                battery_number: 4,
                list_ntc: NtcList::from_list(vec![217, 206, 202,]),
            })
        );
    }

    use super::*;
}

use super::{util::i16_from_bytes, NtcList, ParseError, ParseResult};
