#[derive(Eq, PartialEq, Debug)]
pub struct ProtectionOfState(pub i16);

impl ProtectionOfState {
    pub const NONE: Self = ProtectionOfState(0);
    /// Cell Overvoltage
    pub const COV: Self = ProtectionOfState(1);
    /// Cell Undervoltage
    pub const CUV: Self = ProtectionOfState(2);
    /// Pack Overvoltage
    pub const POV: Self = ProtectionOfState(3);
    /// Pack Undervoltage
    pub const PUV: Self = ProtectionOfState(4);
    /// High-Temp Charging
    pub const OTC: Self = ProtectionOfState(5);
    /// Low-Temp Charging
    pub const UTC: Self = ProtectionOfState(6);
    /// High-Temp Discharging
    pub const OTD: Self = ProtectionOfState(7);
    /// Low-Temp Discharging
    pub const UTD: Self = ProtectionOfState(8);
    /// Over Current Charging
    pub const OCC: Self = ProtectionOfState(9);
    /// Over Current Discharging
    pub const OCD: Self = ProtectionOfState(10);
    /// Short Circuit
    pub const SCD: Self = ProtectionOfState(11);
}
