mod aces;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        eprintln!("no Bluetooth adapters found");
    }

    let adapter = adapters.first().unwrap();

    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let peripherals = adapter.peripherals().await?;
    let battery = find_aces_battery(&peripherals).await.unwrap();

    battery.prepare().await?;

    println!("subscribing for notifications ...");
    let mut notifications = battery.subscribe_notifications().await?;

    println!("requesting SOC ...");
    battery.request_soc().await?;

    println!("waiting for notifications ...");
    if let Some(notification) = notifications.next().await {
        println!("got notification: {:?}", notification);

        let msg = notification.value;
        match aces::Battery::parse_message(&msg) {
            Ok(aces::Message::Soc(soc)) => println!("soc: {}", soc),
            Ok(aces::Message::Detail(detail)) => println!("detail: {:#?}", detail),
            Err(err) => eprintln!("failed to parse message '{:02X?}': {}", msg, err),
        }
    }

    println!("requesting DETAIL ...");
    battery.request_detail().await?;

    println!("waiting for notifications ...");
    if let Some(notification) = notifications.next().await {
        println!("got notification: {:?}", notification);

        let msg = notification.value;
        match aces::Battery::parse_message(&msg) {
            Ok(aces::Message::Soc(soc)) => println!("soc: {}", soc),
            Ok(aces::Message::Detail(detail)) => println!("detail: {:#?}", detail),
            Err(err) => eprintln!("failed to parse message '{:02X?}': {}", msg, err),
        }
    }

    Ok(())
}

async fn find_aces_battery(
    peripherals: &[Peripheral],
) -> Result<aces::Battery, Box<dyn std::error::Error>> {
    for peripheral in peripherals {
        // filter peripherals
        let properties = match peripheral.properties().await? {
            Some(properties) => properties,
            _ => continue,
        };
        if !properties
            .local_name
            .clone()
            .unwrap_or_default()
            .contains("AL12V100HFA0191")
        {
            continue;
        }

        // discover services & characteristics

        if !peripheral.is_connected().await? {
            println!("connecting ...");
            peripheral.connect().await?;
        }

        println!("discovering services ...");
        peripheral.discover_services().await?;

        let characteristics = peripheral.characteristics();
        let _ota = match characteristics
            .iter()
            .find(|char| char.uuid == uuid_from_u16(0xfa01))
        {
            Some(char) => char,
            _ => continue,
        };

        let rx = match characteristics
            .iter()
            .find(|char| char.uuid == uuid_from_u16(0xff01))
        {
            Some(char) => char,
            _ => continue,
        };

        let tx = match characteristics
            .iter()
            .find(|char| char.uuid == uuid_from_u16(0xff02))
        {
            Some(char) => char,
            _ => continue,
        };

        return Ok(aces::Battery::new(
            peripheral.clone(),
            rx.clone(),
            tx.clone(),
        ));
    }

    Err(NotFound.into())
}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
struct NotFound;

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use futures::StreamExt;
use std::time::Duration;
