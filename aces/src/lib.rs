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

pub trait NotificationsReceiver {
    fn next(&mut self) -> Vec<u8>;
}

#[derive(Eq, PartialEq, Debug)]
pub enum ParseError {
    NotEnoughData,
    InvalidChecksum,
    InvalidData,
}

pub async fn read_soc<N>(receiver: &mut N) -> Result<i64>
where
    N: NotificationsReceiver,
{
    log::info!("reading SOC");

    let resp = read_complete_response(receiver);
    match Response::parse_response(&resp) {
        Ok(Response::BatteryVoltage(bv)) => Ok(bv.total()),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn read_detail<N>(receiver: &mut N) -> Result<BatteryDetail>
where
    N: NotificationsReceiver,
{
    log::info!("reading DETAIL");

    let resp = read_complete_response(receiver);
    match Response::parse_response(&resp) {
        Ok(Response::BatteryDetail(detail)) => Ok(detail),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn read_protect<N>(receiver: &mut N) -> Result<BatteryProtect>
where
    N: NotificationsReceiver,
{
    log::info!("reading PROTECT");

    let resp = read_complete_response(receiver);
    match Response::parse_response(&resp) {
        Ok(Response::BatteryProtect(protect)) => Ok(protect),
        _ => Err(WrongNotificationReceived.into()),
    }
}

pub async fn request_soc<F, R, N>(write_value: F, receiver: &mut N) -> Result<i64>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: NotificationsReceiver,
{
    log::info!("requesting SOC");
    write_value(Request::BatteryVoltage.bytes(), false).await?;
    read_soc(receiver).await
}

pub async fn request_detail<F, R, N>(write_value: F, receiver: &mut N) -> Result<BatteryDetail>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: NotificationsReceiver,
{
    log::info!("requesting DETAIL");
    write_value(Request::BatteryDetail.bytes(), false).await?;
    read_detail(receiver).await
}

pub async fn request_protect<F, R, N>(write_value: F, receiver: &mut N) -> Result<BatteryProtect>
where
    F: FnOnce(&[u8], bool) -> R,
    R: Future<Output = Result<()>>,
    N: NotificationsReceiver,
{
    log::info!("requesting PROTECT");
    write_value(Request::BatteryProtect.bytes(), false).await?;
    read_protect(receiver).await
}

fn read_complete_response<N>(receiver: &mut N) -> Vec<u8>
where
    N: NotificationsReceiver,
{
    let mut resp: Vec<u8> = Vec::new();
    while !Response::is_complete_response(&resp) {
        let mut val = receiver.next();
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_complete_response() {
        let mut recv = receiver(vec![
            vec![0x00],
            vec![0x00, 0x00],
            vec![0x00, 0x00, 0x00],
            vec![0x00],
        ]);
        assert_eq!(
            read_complete_response(&mut recv),
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );

        let mut recv = receiver(vec![
            vec![0xdd, 0x04, 0x00, 0x08],
            vec![0x0d, 0xe2, 0x0d, 0xdc, 0x0d],
            vec![0xec, 0x0d, 0xed, 0xfc],
            vec![0x2d, 0x77],
            vec![0x00],
        ]);
        assert_eq!(
            read_complete_response(&mut recv),
            vec![
                0xdd, 0x04, 0x00, 0x08, 0x0d, 0xe2, 0x0d, 0xdc, 0x0d, 0xec, 0x0d, 0xed, 0xfc, 0x2d,
                0x77
            ]
        );
    }

    fn receiver(fragments: Vec<Vec<u8>>) -> Receiver {
        Receiver(VecDeque::from_iter(fragments))
    }

    struct Receiver(VecDeque<Vec<u8>>);
    impl NotificationsReceiver for Receiver {
        fn next(&mut self) -> Vec<u8> {
            self.0.pop_front().unwrap()
        }
    }

    use super::*;
    use std::collections::VecDeque;
}
