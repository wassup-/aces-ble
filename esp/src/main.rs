#![no_main]

mod notifications;

// source: https://github.com/taks/esp32-nimble/blob/develop/examples/ble_client.rs

const SLEEP_DURATION: u64 = 30;
const TARGET_DEVICE_NAME: &str = "AL12V100HFA0191";

esp_idf_sys::esp_app_desc!();

#[no_mangle]
fn app_main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new()).unwrap();

    task::block_on(async {
        let adapter = BLEDevice::take();
        let device = find_target_device(adapter).await;

        let mut client = BLEClient::new();
        connect_to_device(device.addr(), &mut client).await;

        let service = client
            .get_service(Uuid16(aces::SERVICE_UUID))
            .await
            .unwrap();
        let mut characteristics = service.get_characteristics().await.unwrap();
        let rx = characteristics
            .find(|char| char.uuid() == Uuid16(aces::RX_UUID))
            .expect("RX characteristic not found");
        let tx = characteristics
            .find(|char| char.uuid() == Uuid16(aces::TX_UUID))
            .expect("TX characteristic not found");

        let mut notif = notifications::Notifications::subscribe(rx).await;

        tx.write_value(aces::REQ_CLEAR, false).await.unwrap();
        timer.delay(timer.tick_hz()).await.unwrap();

        loop {
            tx.write_value(aces::REQ_BATTERY_VOLTAGE, false)
                .await
                .unwrap();
            let voltage = aces::read_voltage(&mut notif).await.unwrap();
            println!("voltage: {:#?}", voltage);

            tx.write_value(aces::REQ_BATTERY_DETAIL, false)
                .await
                .unwrap();
            let detail = aces::read_detail(&mut notif).await.unwrap();
            println!("detail: {:#?}", detail);

            tx.write_value(aces::REQ_BATTERY_PROTECT, false)
                .await
                .unwrap();
            let protect = aces::read_protect(&mut notif).await.unwrap();
            println!("protect: {:#?}", protect);

            task::do_yield();

            log::info!("sleeping for {} seconds", SLEEP_DURATION);
            timer.delay(timer.tick_hz() * SLEEP_DURATION).await.unwrap();
        }

        // client.disconnect().unwrap();
    });
}

async fn find_target_device(adapter: &BLEDevice) -> BLEAdvertisedDevice {
    log::info!("starting scan ...");

    let scan = adapter.get_scan();
    let device = Arc::new(Mutex::new(None));

    let device0 = device.clone();
    scan.active_scan(true)
        .interval(100)
        .window(99)
        .on_result(move |scan, device| {
            if device.name().contains(TARGET_DEVICE_NAME) {
                scan.stop().unwrap();
                (*device0.lock()) = Some(device.clone());
            }
        });
    scan.start(3000).await.unwrap();

    log::info!("finished scan");

    return match &*device.lock() {
        Some(device) => device.clone(),
        None => {
            log::info!("ACES battery not found");
            panic!("ACES battery not found");
        }
    };
}

async fn connect_to_device(address: &BLEAddress, client: &mut BLEClient) {
    log::info!("connecting to device ...");
    client.on_connect(|client| {
        client.update_conn_params(120, 120, 0, 60).unwrap();
    });
    client.connect(address).await.unwrap();
    log::info!("connected to device");
}

use esp32_nimble::{
    utilities::{mutex::Mutex, BleUuid::Uuid16},
    BLEAddress, BLEAdvertisedDevice, BLEClient, BLEDevice,
};
use esp_idf_hal::{
    prelude::Peripherals,
    task,
    timer::{TimerConfig, TimerDriver},
};
use esp_idf_sys as _;
use std::sync::Arc;
