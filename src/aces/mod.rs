mod detail;
mod message;
mod ntc;
mod protect;
mod util;
mod voltage;

pub use detail::*;
pub use ntc::*;
pub use protect::*;
pub use voltage::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type ParseResult<T> = std::result::Result<T, ParseError>;
type Notifications = Pin<Box<dyn Stream<Item = ValueNotification>>>;

#[derive(Eq, PartialEq, Debug)]
pub enum ParseError {
    NotEnoughData,
    InvalidChecksum,
    InvalidData,
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
        match Message::parse_message(&msg) {
            Ok(Message::Voltage(bv)) => Ok(bv.total()),
            _ => Err(WrongNotificationReceived.into()),
        }
    }

    pub async fn request_detail(&mut self) -> Result<BatteryDetail> {
        log::info!("requesting DETAIL");

        self.write_value(&self.tx, REQ_BATTERY_DETAIL).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let first = self.next_notif_value().await?;
        let mut second = self.next_notif_value().await?;
        let mut msg = first;
        msg.append(&mut second);

        match Message::parse_message(&msg) {
            Ok(Message::Detail(detail)) => Ok(detail),
            _ => Err(WrongNotificationReceived.into()),
        }
    }

    pub async fn request_protect(&mut self) -> Result<BatteryProtect> {
        log::info!("requesting PROTECT");

        self.write_value(&self.tx, REQ_BATTERY_PROTECT).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let first = self.next_notif_value().await?;
        let mut second = self.next_notif_value().await?;
        let mut msg = first;
        msg.append(&mut second);

        match Message::parse_message(&msg) {
            Ok(Message::Protect(protect)) => Ok(protect),
            _ => Err(WrongNotificationReceived.into()),
        }
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
}

// commands

const REQ_CLEAR: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // "00000000000000"
const REQ_BATTERY_DETAIL: &[u8] = &[0xdd, 0xa5, 0x03, 0x00, 0xff, 0xfd, 0x77]; // "DDA50300FFFD77"
const REQ_BATTERY_VOLTAGE: &[u8] = &[0xdd, 0xa5, 0x04, 0x00, 0xff, 0xfc, 0x77]; // "DDA50400FFFC77"
const REQ_BATTERY_PROTECT: &[u8] = &[0xdd, 0xa5, 0xaa, 0x00, 0xff, 0x56, 0x77]; // "DDA5AA00FF5677"

// helpers

#[derive(thiserror::Error, Debug)]
#[error("No notification received")]
struct NoNotificationReceived;

#[derive(thiserror::Error, Debug)]
#[error("Wrong notification received")]
struct WrongNotificationReceived;

use btleplug::api::{
    bleuuid::BleUuid, CharPropFlags, Characteristic, Peripheral as _, ValueNotification, WriteType,
};
use btleplug::platform::Peripheral;
use futures::{Stream, StreamExt};
use message::Message;
use std::pin::Pin;
use std::time::Duration;
