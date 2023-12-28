pub enum Request {
    Clear,
    BatteryDetail,
    BatteryProtect,
    BatteryVoltage,
}

impl Request {
    pub fn bytes(&self) -> &'static [u8] {
        match self {
            Self::Clear => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // "00000000000000"
            Self::BatteryDetail => &[0xdd, 0xa5, 0x03, 0x00, 0xff, 0xfd, 0x77], // "DDA50300FFFD77"
            Self::BatteryProtect => &[0xdd, 0xa5, 0xaa, 0x00, 0xff, 0x56, 0x77], // "DDA5AA00FF5677"
            Self::BatteryVoltage => &[0xdd, 0xa5, 0x04, 0x00, 0xff, 0xfc, 0x77], // "DDA50400FFFC77"
        }
    }
}
