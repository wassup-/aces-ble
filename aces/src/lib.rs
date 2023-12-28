mod checksum;
mod detail;
mod ntc;
mod protect;
mod request;
mod response;
mod util;
mod voltage;

pub use checksum::*;
pub use detail::*;
pub use ntc::*;
pub use protect::*;
pub use request::*;
pub use response::*;
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

    let resp = read_complete_response(notif);
    match Response::parse_response(&resp) {
        Ok(Response::BatteryVoltage(bv)) => Ok(bv.total()),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn read_detail<N>(notif: &mut N) -> Result<BatteryDetail>
where
    N: Notifications,
{
    log::info!("reading DETAIL");

    let resp = read_complete_response(notif);
    match Response::parse_response(&resp) {
        Ok(Response::BatteryDetail(detail)) => Ok(detail),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn read_protect<N>(notif: &mut N) -> Result<BatteryProtect>
where
    N: Notifications,
{
    log::info!("reading PROTECT");

    let resp = read_complete_response(notif);
    match Response::parse_response(&resp) {
        Ok(Response::BatteryProtect(protect)) => Ok(protect),
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
    write_value(Request::BatteryVoltage.bytes(), false).await?;
    read_soc(notif).await
}

pub async fn request_detail<F, R, N>(write_value: F, notif: &mut N) -> Result<BatteryDetail>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: Notifications,
{
    log::info!("requesting DETAIL");
    write_value(Request::BatteryDetail.bytes(), false).await?;
    read_detail(notif).await
}

pub async fn request_protect<F, R, N>(write_value: F, notif: &mut N) -> Result<BatteryProtect>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: Notifications,
{
    log::info!("requesting PROTECT");
    write_value(Request::BatteryProtect.bytes(), false).await?;
    read_protect(notif).await
}

fn read_complete_response<N>(notif: &mut N) -> Vec<u8>
where
    N: Notifications,
{
    let mut resp: Vec<u8> = Vec::new();
    while !Response::is_complete_response(&resp) {
        let mut val = notif.next();
        resp.append(&mut val);
    }
    resp
}

// commands

pub const SERVICE_UUID: u16 = 0xff00;
pub const RX_UUID: u16 = 0xff01;
pub const TX_UUID: u16 = 0xff02;

// helpers

#[derive(thiserror::Error, Debug)]
#[error("Wrong notification received")]
struct WrongNotificationReceived;

use std::future::Future;
