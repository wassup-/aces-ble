pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Notifications = Pin<Box<dyn Stream<Item = ValueNotification>>>;

pub enum Message {
    Soc(i64),
    Detail(BatteryDetail),
}

#[derive(Debug)]
pub struct Battery {
    pub peripheral: Peripheral,
    pub rx: Characteristic,
    tx: Characteristic,
}

impl Battery {
    pub fn new(peripheral: Peripheral, rx: Characteristic, tx: Characteristic) -> Self {
        Battery { peripheral, rx, tx }
    }

    pub async fn prepare(&self) -> Result<()> {
        self.read_value(&self.rx).await?;
        self.read_value(&self.tx).await?;
        self.write_value(&self.tx, REQ_BATTERY_DETAIL).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    pub async fn subscribe_notifications(&self) -> Result<Notifications> {
        for characteristic in self.peripheral.characteristics() {
            if characteristic.properties.contains(CharPropFlags::NOTIFY) {
                log::debug!("subscribing to characteristic {} ...", characteristic.uuid);
                self.peripheral.subscribe(&characteristic).await?;
            }
        }

        let notif = self.peripheral.notifications().await?;
        Ok(notif)
    }

    pub async fn request_soc(&self) -> Result<()> {
        self.write_value(&self.tx, REQ_BATTERY_VOLTAGE).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    pub async fn request_detail(&self) -> Result<()> {
        self.write_value(&self.tx, REQ_BATTERY_DETAIL).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    async fn write_value(&self, characteristic: &Characteristic, value: &[u8]) -> Result<()> {
        log::info!(
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
        log::info!("reading from {} ...", characteristic.uuid.to_short_string());

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

    pub fn parse_message(msg: &[u8]) -> Result<Message> {
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
            todo!()
        }

        Err(ParseMessageFailed.into())
    }
}

// commands

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
    // pub list_ntc: i64,
    // pub temperature: i64,
}

// public static BatteryDetailBean getBatteryDetailBean(String str) {
//     String str2;
//     BatteryDetailBean batteryDetailBean = new BatteryDetailBean();
//     batteryDetailBean.setData(str);
//     batteryDetailBean.setInstDate(C1165k.m2225a());
//     byte[] c = C1162h.m2220c(str);
//     str.length();
//     batteryDetailBean.setTotalVoltage(C1171q.m2262o(((double) C1171q.m2255h(str.substring(8, 12))) / 100.0d, 1) + "");
//     double h = ((double) C1171q.m2255h(str.substring(12, 16))) / 100.0d;
//     if (h > 327.68d) {
//         h -= 655.36d;
//     }
//     batteryDetailBean.setCurrent(C1171q.m2262o(h, 1) + "");
//     batteryDetailBean.setResidualCapacity(C1171q.m2262o(((double) C1171q.m2255h(str.substring(16, 20))) / 100.0d, 1) + "");
//     batteryDetailBean.setStandarCapacity(C1171q.m2262o(((double) C1171q.m2255h(str.substring(20, 24))) / 100.0d, 1) + "");
//     batteryDetailBean.setCycles(C1171q.m2255h(str.substring(24, 28)) + "");
//     long h2 = C1171q.m2255h(str.substring(28, 32));
//     batteryDetailBean.setDateOfProduction(((h2 >> 9) + ItemTouchHelper.Callback.DRAG_SCROLL_ACCELERATION_LIMIT_TIME_MS) + "-" + ((h2 >> 5) & 15));
//     String substring = str.substring(32, 36);
//     batteryDetailBean.setEquilibrium(substring);
//     String substring2 = str.substring(36, 40);
//     batteryDetailBean.setEquilibriumHigh(substring2);
//     boolean[] a = m539a(substring);
//     boolean[] a2 = m539a(substring2);
//     boolean[] zArr = new boolean[(a.length + a2.length)];
//     for (int i = 0; i < a.length; i++) {
//         zArr[i] = a[i];
//     }
//     for (int i2 = 0; i2 < a2.length; i2++) {
//         zArr[i2 + 16] = a2[i2];
//     }
//     batteryDetailBean.setBalanceStates(zArr);
//     batteryDetailBean.setProtectionOfState(C1171q.m2256i(str.substring(40, 44)));
//     batteryDetailBean.setSoftwareVersion(str.substring(44, 46));
//     batteryDetailBean.setResidualCapacityPercentage(C1171q.m2255h(str.substring(46, 48)) + "");
//     batteryDetailBean.setControlState(str.substring(48, 50));
//     byte b = C1162h.m2220c(str)[24];
//     batteryDetailBean.setCharge((b & 1) == 1);
//     batteryDetailBean.setDisCharge((b & 2) == 2);
//     batteryDetailBean.setBatteryNumber(C1171q.m2261n(str.substring(50, 52)));
//     String substring3 = str.substring(52, 54);
//     batteryDetailBean.setNumberNTC(substring3);
//     batteryDetailBean.setListNTC(m540b(batteryDetailBean, c));
//     if (C1157d.m2201c(substring3) > 0) {
//         str2 = C1171q.m2262o((double) batteryDetailBean.getListNTC().get(0).floatValue(), 1) + "";
//     } else {
//         str2 = "0";
//     }
//     batteryDetailBean.setTemperature(str2);
//     return batteryDetailBean;
// }

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
    if msg.len() < 18 {
        return None;
    }

    let msg = &msg[2..];
    Some(BatteryDetail {
        total_voltage: i16_from_bytes(&msg[0..2]),
        current: i16_from_bytes(&msg[2..4]),
        residual_capacity: i16_from_bytes(&msg[4..6]),
        standard_capacity: i16_from_bytes(&msg[6..8]),
        cycles: i16_from_bytes(&msg[8..10]),
        date_of_production: i16_from_bytes(&msg[10..12]),
        equilibrium: i16_from_bytes(&msg[12..14]),
        equilibrium_high: i16_from_bytes(&msg[14..16]),
        // TODO:
        protection_of_state: 0,
        software_version: 0,
        residual_capacity_percent: 0,
        control_state: 0,
        charge: false,
        discharge: false,
        battery_number: 0,
        number_ntc: 0,
        // protection_of_state: i16_from_bytes(&msg[18..20]),
        // software_version: msg[20],
        // residual_capacity_percent: msg[21],
        // control_state: msg[22],
        // charge: (msg[22] & 1) == 1,
        // discharge: (msg[22] & 2) == 2,
        // battery_number: msg[23],
        // number_ntc: msg[24],
        // list_ntc: i16_from_bytes(&msg[4..6]),
        // temperature: i16_from_bytes(&msg[4..6]),
    })
}

fn i16_from_bytes(b: &[u8]) -> i16 {
    assert!(b.len() == 2);
    // device uses big endian encoding
    i16::from_be_bytes([b[0], b[1]])
}

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
                0, 29, 5, 52, 0, 0, 38, 160, 39, 222, 0, 10, 43, 148, 0, 0, 0, 0
            ]),
            Some(BatteryDetail {
                total_voltage: 1332,
                current: 0,
                residual_capacity: 9888,
                standard_capacity: 10206,
                cycles: 10,
                date_of_production: 11156,
                equilibrium: 0,
                equilibrium_high: 0,
                protection_of_state: 0,
                software_version: 0,
                residual_capacity_percent: 0,
                control_state: 0,
                charge: false,
                discharge: false,
                battery_number: 0,
                number_ntc: 0,
            })
        );

        assert_eq!(
            parse_battery_detail(&[
                0, 29, 5, 53, 5, 182, 38, 48, 39, 222, 0, 10, 43, 148, 0, 0, 0, 0
            ]),
            Some(BatteryDetail {
                total_voltage: 1333,
                current: 1462,
                residual_capacity: 9776,
                standard_capacity: 10206,
                cycles: 10,
                date_of_production: 11156,
                equilibrium: 0,
                equilibrium_high: 0,
                protection_of_state: 0,
                software_version: 0,
                residual_capacity_percent: 0,
                control_state: 0,
                charge: false,
                discharge: false,
                battery_number: 0,
                number_ntc: 0,
            })
        );
    }

    use super::*;
}

use btleplug::api::{
    bleuuid::BleUuid, CharPropFlags, Characteristic, Peripheral as _, ValueNotification, WriteType,
};
use btleplug::platform::Peripheral;
use futures::Stream;
use std::pin::Pin;
use std::time::Duration;
