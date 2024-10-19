mod notifications;

const SLEEP_DURATION: u64 = 30;
/// The name of the target device (e.g. the device to connect to).
const TARGET_DEVICE_NAME: &str = "AL12V100HFA0191";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        eprintln!("no Bluetooth adapters found");
    }

    let adapter = adapters.first().unwrap();
    let peripheral = find_target_device(adapter).await?;
    let (tx, rx) = connect_to_device(&peripheral).await?;

    let mut notif = notifications::Notifications::subscribe(&peripheral, rx).await?;

    peripheral
        .write(
            &tx,
            aces::Request::Clear.bytes(),
            WriteType::WithoutResponse,
        )
        .await?;

    tokio::time::sleep(Duration::from_secs(1)).await;
    // clear any stale notifications
    notif.clear();

    loop {
        println!("local time: {}", chrono::Local::now().to_rfc3339());

        peripheral
            .write(
                &tx,
                aces::Request::BatteryVoltage.bytes(),
                WriteType::WithoutResponse,
            )
            .await?;
        let voltage = aces::read_voltage(&mut notif).await?;
        println!("voltage: {:#?}", voltage);

        peripheral
            .write(
                &tx,
                aces::Request::BatteryDetail.bytes(),
                WriteType::WithoutResponse,
            )
            .await?;
        let detail = aces::read_detail(&mut notif).await?;
        println!("detail: {:#?}", detail);

        peripheral
            .write(
                &tx,
                aces::Request::BatteryProtect.bytes(),
                WriteType::WithoutResponse,
            )
            .await?;
        let protect = aces::read_protect(&mut notif).await?;
        println!("protect: {:#?}", protect);

        log::info!("sleeping for {} seconds", SLEEP_DURATION);
        tokio::time::sleep(Duration::from_secs(SLEEP_DURATION)).await;
        println!();
    }
}

async fn find_target_device(adapter: &Adapter) -> Result<Peripheral> {
    log::info!("starting scan ...");

    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    let peripherals = adapter.peripherals().await?;

    log::info!("finished scan");

    for peripheral in peripherals {
        // filter peripherals
        let properties = match peripheral.properties().await {
            Ok(Some(properties)) => properties,
            _ => continue,
        };
        if !properties
            .local_name
            .clone()
            .unwrap_or_default()
            .contains(TARGET_DEVICE_NAME)
        {
            continue;
        }

        return Ok(peripheral);
    }

    log::info!("ACES battery not found");
    Err(NotFound.into())
}

async fn connect_to_device(peripheral: &Peripheral) -> Result<(Characteristic, Characteristic)> {
    if !peripheral.is_connected().await? {
        log::info!("connecting to device ...");
        peripheral.connect().await?;
    }

    log::info!("connected to device");

    log::info!("discovering services ...");
    peripheral.discover_services().await?;

    let characteristics = peripheral.characteristics();

    let rx = match characteristics
        .iter()
        .find(|char| char.uuid == uuid_from_u16(0xff01))
    {
        Some(char) => char,
        _ => return Err(NotFound.into()),
    };

    let tx = match characteristics
        .iter()
        .find(|char| char.uuid == uuid_from_u16(0xff02))
    {
        Some(char) => char,
        _ => return Err(NotFound.into()),
    };

    Ok((tx.clone(), rx.clone()))
}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
struct NotFound;

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::time::Duration;
