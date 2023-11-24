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

pub trait Notifications {
    fn next(&mut self) -> Vec<u8>;
}

#[derive(Eq, PartialEq, Debug)]
pub enum ParseError {
    NotEnoughData,
    InvalidChecksum,
    InvalidData,
}

pub async fn read_soc<N>(notif: &mut N) -> Result<i64>
where
    N: Notifications,
{
    log::info!("reading SOC");

    let msg = read_complete_message(notif);
    match Message::parse_message(&msg) {
        Ok(Message::Voltage(bv)) => Ok(bv.total()),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn read_detail<N>(notif: &mut N) -> Result<BatteryDetail>
where
    N: Notifications,
{
    log::info!("reading DETAIL");

    let msg = read_complete_message(notif);
    match Message::parse_message(&msg) {
        Ok(Message::Detail(detail)) => Ok(detail),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn read_protect<N>(notif: &mut N) -> Result<BatteryProtect>
where
    N: Notifications,
{
    log::info!("reading PROTECT");

    let msg = read_complete_message(notif);
    match Message::parse_message(&msg) {
        Ok(Message::Protect(protect)) => Ok(protect),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn request_soc<F, R, N>(write_value: F, notif: &mut N) -> Result<i64>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: Notifications,
{
    log::info!("requesting SOC");
    write_value(REQ_BATTERY_VOLTAGE, false).await?;
    read_soc(notif).await
}

pub async fn request_detail<F, R, N>(write_value: F, notif: &mut N) -> Result<BatteryDetail>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: Notifications,
{
    log::info!("requesting DETAIL");
    write_value(REQ_BATTERY_DETAIL, false).await?;
    read_detail(notif).await
}

pub async fn request_protect<F, R, N>(write_value: F, notif: &mut N) -> Result<BatteryProtect>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: Notifications,
{
    log::info!("requesting PROTECT");
    write_value(REQ_BATTERY_PROTECT, false).await?;
    read_protect(notif).await
}

fn read_complete_message<N>(notif: &mut N) -> Vec<u8>
where
    N: Notifications,
{
    let mut msg: Vec<u8> = Vec::new();
    while !Message::is_complete_message(&msg) {
        let mut val = notif.next();
        msg.append(&mut val);
    }
    msg
}

// commands

pub const SERVICE_UUID: u16 = 0xff00;
pub const RX_UUID: u16 = 0xff01;
pub const TX_UUID: u16 = 0xff02;

pub const REQ_CLEAR: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // "00000000000000"
pub const REQ_BATTERY_DETAIL: &[u8] = &[0xdd, 0xa5, 0x03, 0x00, 0xff, 0xfd, 0x77]; // "DDA50300FFFD77"
pub const REQ_BATTERY_VOLTAGE: &[u8] = &[0xdd, 0xa5, 0x04, 0x00, 0xff, 0xfc, 0x77]; // "DDA50400FFFC77"
pub const REQ_BATTERY_PROTECT: &[u8] = &[0xdd, 0xa5, 0xaa, 0x00, 0xff, 0x56, 0x77]; // "DDA5AA00FF5677"

// helpers

#[derive(thiserror::Error, Debug)]
#[error("Wrong notification received")]
struct WrongNotificationReceived;

use message::Message;
use std::future::Future;
