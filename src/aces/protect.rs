#[derive(Eq, PartialEq, Debug, Default)]
pub struct BatteryProtect {
    pub short_circuit: i16,
    pub over_current_charging: i16,
    pub over_current_discharging: i16,
    pub cell_overvoltage: i16,
    pub cell_undervoltage: i16,
    pub high_temp_charging: i16,
    pub low_temp_charging: i16,
    pub high_temp_discharging: i16,
    pub low_temp_discharging: i16,
    pub pack_overvoltage: i16,
    pub pack_undervoltage: i16,
}

impl BatteryProtect {
    pub fn set_value_at(&mut self, idx: usize, value: i16) {
        match idx {
            0 => self.short_circuit = value,
            1 => self.over_current_charging = value,
            2 => self.over_current_discharging = value,
            3 => self.cell_overvoltage = value,
            4 => self.cell_undervoltage = value,
            5 => self.high_temp_charging = value,
            6 => self.low_temp_charging = value,
            7 => self.high_temp_discharging = value,
            8 => self.low_temp_discharging = value,
            9 => self.pack_overvoltage = value,
            10 => self.pack_undervoltage = value,
            _ => (),
        }
    }

    pub fn parse_message(msg: &[u8]) -> ParseResult<BatteryProtect> {
        if msg.len() < 11 {
            return Err(ParseError::NotEnoughData);
        }

        let mut protect = BatteryProtect::default();
        for i in 0..11 {
            protect.set_value_at(i, i16_from_bytes(&msg[i..(i + 2)]))
        }
        Ok(protect)
    }
}

use super::{util::i16_from_bytes, ParseError, ParseResult};
