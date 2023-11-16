mod checksum;
mod detail;
mod message;
mod ntc;
mod protect;
mod util;
mod voltage;

pub use checksum::*;
pub use detail::*;
pub use ntc::*;
pub use protect::*;
pub use voltage::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Eq, PartialEq, Debug)]
pub enum ParseError {
    NotEnoughData,
    InvalidChecksum,
    InvalidData,
}

pub async fn request_soc(
    tx: &mut BLERemoteCharacteristic,
    notif: &mut Notifications,
) -> Result<i64> {
    log::info!("requesting SOC");

    tx.write_value(REQ_BATTERY_VOLTAGE, false).await.unwrap();

    let msg = notif.next();
    match Message::parse_message(&msg) {
        Ok(Message::Voltage(bv)) => Ok(bv.total()),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn request_detail(
    tx: &mut BLERemoteCharacteristic,
    notif: &mut Notifications,
) -> Result<BatteryDetail> {
    log::info!("requesting DETAIL");

    tx.write_value(REQ_BATTERY_DETAIL, false).await.unwrap();

    let first = notif.next();
    let mut second = notif.next();
    let mut msg = first;
    msg.append(&mut second);

    match Message::parse_message(&msg) {
        Ok(Message::Detail(detail)) => Ok(detail),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn request_protect(
    tx: &mut BLERemoteCharacteristic,
    notif: &mut Notifications,
) -> Result<BatteryProtect> {
    log::info!("requesting PROTECT");

    tx.write_value(REQ_BATTERY_PROTECT, false).await.unwrap();

    let first = notif.next();
    let mut second = notif.next();
    let mut msg = first;
    msg.append(&mut second);

    match Message::parse_message(&msg) {
        Ok(Message::Protect(protect)) => Ok(protect),
        _ => Err(WrongNotificationReceived.into()),
    }
}

// commands

pub const SERVICE_UUID: u16 = 0xff00;
pub const RX_UUID: u16 = 0xff01;
pub const TX_UUID: u16 = 0xff02;

pub const REQ_CLEAR: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // "00000000000000"
pub const REQ_BATTERY_DETAIL: &[u8] = &[0xdd, 0xa5, 0x03, 0x00, 0xff, 0xfd, 0x77]; // "DDA50300FFFD77"
pub const REQ_BATTERY_VOLTAGE: &[u8] = &[0xdd, 0xa5, 0x04, 0x00, 0xff, 0xfc, 0x77]; // "DDA50400FFFC77"
pub const REQ_BATTERY_PROTECT: &[u8] = &[0xdd, 0xa5, 0xaa, 0x00, 0xff, 0x56, 0x77]; // "DDA5AA00FF5677"

pub struct Notifications {
    notif: Arc<Mutex<VecDeque<Vec<u8>>>>,
    cv: Arc<Condvar>,
}

impl Notifications {
    pub async fn subscribe(characteristic: &mut BLERemoteCharacteristic) -> Self {
        let notif = Arc::new(Mutex::new(VecDeque::new()));
        let cv = Arc::new(Condvar::new());

        let notif0 = Arc::clone(&notif);
        let cv0 = Arc::clone(&cv);
        characteristic
            .on_notify(move |value| {
                log::info!("got notification");
                let mut lock = notif0.lock();
                lock.push_back(value.to_vec());
                cv0.notify_one();
            })
            .subscribe_notify(false)
            .await
            .unwrap();

        Notifications { notif, cv }
    }

    pub fn next(&mut self) -> Vec<u8> {
        log::info!("awaiting next notification");

        let mut lock = self.notif.lock();
        // protect agains spurious wake-ups
        while lock.is_empty() {
            lock = self.cv.wait(lock);
        }

        let val = lock.pop_front().unwrap();
        self.cv.notify_one();
        val
    }
}

// helpers

#[derive(thiserror::Error, Debug)]
#[error("No notification received")]
struct NoNotificationReceived;

#[derive(thiserror::Error, Debug)]
#[error("Wrong notification received")]
struct WrongNotificationReceived;

use esp32_nimble::{
    utilities::mutex::{Condvar, Mutex},
    BLERemoteCharacteristic, BLEReturnCode,
};
use esp_idf_hal::timer::TimerDriver;
use message::Message;
use std::{collections::VecDeque, sync::Arc};
