pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Notifications = Pin<Box<dyn Stream<Item = ValueNotification>>>;

enum Message {
    Soc(i64),
    Detail(BatteryDetail),
    Protect(BatteryProtect),
}

pub struct Battery {
    pub peripheral: Peripheral,
    pub rx: Characteristic,
    tx: Characteristic,
    notif: Notifications,
}

impl Battery {
    pub async fn prepare(
        peripheral: Peripheral,
        rx: Characteristic,
        tx: Characteristic,
    ) -> Result<Self> {
        let notif = peripheral.notifications().await?;
        let batt = Battery {
            peripheral,
            rx,
            tx,
            notif,
        };

        batt.subscribe_notifications().await?;
        batt.write_value(&batt.tx, REQ_CLEAR).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(batt)
    }

    async fn subscribe_notifications(&self) -> Result<()> {
        for characteristic in self.peripheral.characteristics() {
            if characteristic.properties.contains(CharPropFlags::NOTIFY) {
                log::debug!("subscribing to characteristic {} ...", characteristic.uuid);
                self.peripheral.subscribe(&characteristic).await?;
            }
        }

        Ok(())
    }

    pub async fn request_soc(&mut self) -> Result<i64> {
        log::info!("requesting SOC");

        self.write_value(&self.tx, REQ_BATTERY_VOLTAGE).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let msg = self.next_notif_value().await?;
        return match Battery::parse_message(&msg) {
            Ok(Message::Soc(soc)) => Ok(soc),
            _ => Err(WrongNotificationReceived.into()),
        };
    }

    pub async fn request_detail(&mut self) -> Result<BatteryDetail> {
        log::info!("requesting DETAIL");

        self.write_value(&self.tx, REQ_BATTERY_DETAIL).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let first = self.next_notif_value().await?;
        let mut second = self.next_notif_value().await?;
        let mut msg = first;
        msg.append(&mut second);

        return match Battery::parse_message(&msg) {
            Ok(Message::Detail(detail)) => Ok(detail),
            _ => Err(WrongNotificationReceived.into()),
        };
    }

    pub async fn request_protect(&mut self) -> Result<BatteryProtect> {
        log::info!("requesting PROTECT");

        self.write_value(&self.tx, REQ_BATTERY_PROTECT).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let msg = self.next_notif_value().await?;
        return match Battery::parse_message(&msg) {
            Ok(Message::Protect(protect)) => Ok(protect),
            _ => Err(WrongNotificationReceived.into()),
        };
    }

    async fn write_value(&self, characteristic: &Characteristic, value: &[u8]) -> Result<()> {
        log::debug!(
            "writing {:02X?} to {}",
            value,
            characteristic.uuid.to_short_string()
        );

        self.peripheral
            .write(characteristic, value, WriteType::WithoutResponse)
            .await?;
        Ok(())
    }

    async fn read_value(&self, characteristic: &Characteristic) -> Result<Vec<u8>> {
        match self.peripheral.read(characteristic).await {
            Ok(value) => {
                log::debug!(
                    "> read {:02X?} from {}",
                    value,
                    characteristic.uuid.to_short_string()
                );
                Ok(value)
            }
            Err(err) => {
                log::error!(
                    "> failed to read value from {}",
                    characteristic.uuid.to_short_string()
                );
                Err(err.into())
            }
        }
    }

    async fn next_notif_value(&mut self) -> Result<Vec<u8>> {
        let notif = self.notif.next().await.ok_or(NoNotificationReceived)?;
        log::debug!("received notification {:02X?}", notif.value);
        Ok(notif.value)
    }

    fn parse_message(msg: &[u8]) -> Result<Message> {
        if let Some(msg) = msg.strip_prefix(&[0xdd, 0x03]) {
            return match parse_battery_detail(msg) {
                Some(detail) => Ok(Message::Detail(detail)),
                None => Err(ParseMessageFailed.into()),
            };
        } else if let Some(msg) = msg.strip_prefix(&[0xdd, 0x04]) {
            return match parse_battery_voltage(msg) {
                Some(voltage) => Ok(Message::Soc(voltage.total())),
                None => Err(ParseMessageFailed.into()),
            };
        } else if let Some(msg) = msg.strip_prefix(&[0xdd, 0xaa]) {
            return match parse_battery_protect(msg) {
                Some(protect) => Ok(Message::Protect(protect)),
                None => Err(ParseMessageFailed.into()),
            };
        }

        Err(ParseMessageFailed.into())
    }
}

// commands

const REQ_CLEAR: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // "00000000000000"
const REQ_BATTERY_DETAIL: &[u8] = &[0xdd, 0xa5, 0x03, 0x00, 0xff, 0xfd, 0x77]; // "DDA50300FFFD77"
const REQ_BATTERY_VOLTAGE: &[u8] = &[0xdd, 0xa5, 0x04, 0x00, 0xff, 0xfc, 0x77]; // "DDA50400FFFC77"
const REQ_BATTERY_PROTECT: &[u8] = &[0xdd, 0xa5, 0xaa, 0x00, 0xff, 0x56, 0xff]; // "DDA5AA00FF5677"

// helpers

#[derive(Eq, PartialEq, Debug)]
struct BatteryVoltage(i16, i16, i16, i16);

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
    pub number_ntc: u8,
    pub list_ntc: Vec<i16>,
    pub temperature: i16,
}

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

impl BatteryVoltage {
    /// The total voltage.
    pub fn total(&self) -> i64 {
        self.0 as i64 + self.1 as i64 + self.2 as i64 + self.3 as i64
    }

    /// The average voltage.
    pub fn avg(&self) -> i64 {
        self.total() / 4
    }

    /// The minimum voltage.
    pub fn min(&self) -> i64 {
        self.0.min(self.1).min(self.2).min(self.3) as i64
    }

    /// The maximum voltage.
    pub fn max(&self) -> i64 {
        self.0.max(self.1).max(self.2).max(self.3) as i64
    }
}

impl BatteryProtect {
    fn set_value_at(&mut self, idx: usize, value: i16) {
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
}

fn parse_battery_voltage(msg: &[u8]) -> Option<BatteryVoltage> {
    if msg.len() < 10 {
        return None;
    }

    let msg = &msg[2..10];
    Some(BatteryVoltage(
        i16_from_bytes(&msg[0..2]),
        i16_from_bytes(&msg[2..4]),
        i16_from_bytes(&msg[4..6]),
        i16_from_bytes(&msg[6..8]),
    ))
}

fn parse_battery_detail(msg: &[u8]) -> Option<BatteryDetail> {
    if msg.len() < 22 {
        return None;
    }

    let msg = &msg[2..];
    let list_ntc = parse_list_ntc(&msg[23..], msg[22] as usize);
    let temperature = *list_ntc.first().unwrap_or(&0);

    Some(BatteryDetail {
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
        number_ntc: msg[22],
        list_ntc,
        temperature,
    })
}

fn parse_list_ntc(b: &[u8], num_ntc: usize) -> Vec<i16> {
    assert!(b.len() >= num_ntc * 2);

    let mut ntc = Vec::new();
    for i in 0..num_ntc {
        let offset = i * 2;
        ntc.push(i16_from_bytes(&b[offset..(offset + 2)]) - 2731);
    }
    ntc
}

fn parse_battery_protect(msg: &[u8]) -> Option<BatteryProtect> {
    if msg.len() < 11 {
        return None;
    }

    let mut protect = BatteryProtect::default();
    for i in 0..11 {
        protect.set_value_at(i, i16_from_bytes(&msg[i..(i + 2)]))
    }
    Some(protect)
}

fn i16_from_bytes(b: &[u8]) -> i16 {
    assert!(b.len() == 2);
    // device uses big endian encoding
    i16::from_be_bytes([b[0], b[1]])
}

#[derive(thiserror::Error, Debug)]
#[error("No notification received")]
struct NoNotificationReceived;

#[derive(thiserror::Error, Debug)]
#[error("Wrong notification received")]
struct WrongNotificationReceived;

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse message")]
struct ParseMessageFailed;

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_battery_voltage() {
        assert_eq!(
            parse_battery_voltage(&[
                0x00, 0x08, 0x0d, 0x0b, 0x0d, 0x0d, 0x0d, 0x0b, 0x0d, 0x0f, 0xff, 0x92, 0x77
            ]),
            Some(BatteryVoltage(3339, 3341, 3339, 3343))
        );
    }

    #[test]
    fn test_parse_battery_detail() {
        assert_eq!(
            parse_battery_detail(&[
                0x00, 0x1D, 0x05, 0x35, 0x00, 0x00, 0x24, 0xB7, 0x27, 0xDE, 0x00, 0x0A, 0x2B, 0x94,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x5C, 0x03, 0x04, 0x03, 0x0B, 0x84, 0x0B,
                0x79, 0x0B, 0x75, 0xFA, 0xE7, 0x77
            ]),
            Some(BatteryDetail {
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
                number_ntc: 3,
                list_ntc: vec![217, 206, 202,],
                temperature: 217,
            })
        );
    }

    use super::*;
}

use btleplug::api::{
    bleuuid::BleUuid, CharPropFlags, Characteristic, Peripheral as _, ValueNotification, WriteType,
};
use btleplug::platform::Peripheral;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::time::Duration;
