#![no_main]

mod channel;

// source: https://github.com/taks/esp32-nimble/blob/develop/examples/ble_client.rs

/// The name of the target device (e.g. the device to connect to).
const DEVICE_NAME: &str = "ACES-MOCK";

esp_idf_sys::esp_app_desc!();

#[no_mangle]
fn app_main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let adapter = BLEDevice::take();
    let (mut tx, mut rx) = setup_ble_device(&adapter, DEVICE_NAME);

    loop {
        if let Some(req) = read_request(&mut rx) {
            log::info!("received request {:?}", req);
            write_response(&mut tx, req);
        }

        task::do_yield();
    }
}

fn setup_ble_device<'a>(adapter: &'a BLEDevice, name: &str) -> channel::Channel {
    log::info!("setting up BLE device ...");

    let advertising = adapter.get_advertising();
    let server = adapter.get_server();

    server.on_connect(|server, desc| {
        log::info!("client connected ({:?})", desc);

        server
            .update_conn_params(desc.conn_handle, 120, 120, 0, 60)
            .expect("failed to update conn params");
    });

    server.on_disconnect(|_desc, reason| {
        log::info!("client disconnected ({:?})", reason);
    });

    let service = server.create_service(BleUuid::from_uuid16(::aces::SERVICE_UUID));

    let rx = service.lock().create_characteristic(
        BleUuid::from_uuid16(::aces::TX_UUID),
        NimbleProperties::WRITE,
    );
    let tx = service.lock().create_characteristic(
        BleUuid::from_uuid16(::aces::RX_UUID),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );

    advertising.name(name);
    advertising.add_service_uuid(BleUuid::from_uuid16(::aces::SERVICE_UUID));

    advertising.start().expect("failed to start advertising");

    channel::channel(tx, rx)
}

fn read_request(rx: &mut channel::Receiver) -> Option<::aces::Request> {
    log::info!("reading request ...");

    let mut buff = Vec::new();

    while !::aces::Request::is_complete_request(&buff) {
        let mut chunk = rx.recv();
        log::debug!("read chunk {:x?}", chunk);
        buff.append(&mut chunk);
    }

    log::debug!("parsing request {:x?}", buff);
    ::aces::Request::parse_request(&buff).ok()
}

fn write_response(tx: &mut channel::Sender, req: ::aces::Request) {
    log::info!("writing response ...");

    let res = match req {
        ::aces::Request::Clear => return,
        ::aces::Request::BatteryDetail => vec![
            0xdd, 0x03, 0x00, 0x1d, 0x05, 0x38, 0x02, 0x83, 0x17, 0x5c, 0x27, 0xde, 0x00, 0x09,
            0x2b, 0x94, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x3b, 0x03, 0x04, 0x03, 0x0b,
            0x7f, 0x0b, 0x6c, 0x0b, 0x69, 0xfb, 0x07, 0x77,
        ],
        ::aces::Request::BatteryVoltage => vec![
            0xdd, 0x04, 0x00, 0x08, 0x0d, 0xe2, 0x0d, 0xdc, 0x0d, 0xec, 0x0d, 0xed, 0xfc, 0x2d,
            0x77,
        ],
        ::aces::Request::BatteryProtect => vec![
            0xdd, 0xaa, 0x00, 0x16, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xe6,
            0x77,
        ],
    };

    log::info!("sending response {:x?}", res);
    tx.send(&res);
}

use esp32_nimble::{utilities::BleUuid, BLEDevice, NimbleProperties};
use esp_idf_hal::task;
use esp_idf_sys as _;
