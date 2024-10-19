#![no_main]

mod channel;

// source: https://github.com/taks/esp32-nimble/blob/develop/examples/ble_client.rs

/// The advertised name of the device.
const DEVICE_NAME: &str = "ACES-MOCK";

esp_idf_sys::esp_app_desc!();

enum StateMachine {
    Disconnected,
    Notify(::aces::Request),
    ReadRequest(Vec<u8>),
    WriteResponse(Vec<u8>),
}

#[no_mangle]
fn app_main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut device = device::setup_ble_device(DEVICE_NAME);

    let mut state = StateMachine::Disconnected;

    let mut counter = 0;

    loop {
        log::trace!("counter: {}, state: {:?}", counter, state);

        if !device.is_connected() {
            state = StateMachine::Disconnected;
        }

        state = match state {
            StateMachine::Disconnected => fsm::state_disconnected(&device),
            StateMachine::Notify(req) => fsm::state_notify(&mut device, req),
            StateMachine::ReadRequest(buff) => fsm::state_read_request(&mut device, buff),
            StateMachine::WriteResponse(resp) => fsm::state_write_response(&mut device, resp),
        };

        counter += 1;

        task::do_yield();
    }
}

mod fsm {
    pub fn state_disconnected(device: &device::Device) -> StateMachine {
        esp_idf_hal::delay::FreeRtos::delay_ms(10);

        if device.is_connected() {
            StateMachine::Notify(::aces::Request::BatteryDetail)
        } else {
            StateMachine::Disconnected
        }
    }

    pub fn state_notify(device: &mut device::Device, req: ::aces::Request) -> StateMachine {
        if let Some(chunk) = device.try_recv() {
            // clear the response
            device.set_response(&[]);

            StateMachine::ReadRequest(chunk)
        } else {
            let resp = device::response_for_request(&req);
            device.set_response(&resp);

            esp_idf_hal::delay::FreeRtos::delay_ms(500);

            let req = match req {
                ::aces::Request::BatteryDetail => ::aces::Request::BatteryProtect,
                ::aces::Request::BatteryProtect => ::aces::Request::BatteryVoltage,
                ::aces::Request::BatteryVoltage => ::aces::Request::BatteryDetail,
                _ => unreachable!(),
            };
            StateMachine::Notify(req)
        }
    }

    pub fn state_read_request(device: &mut device::Device, mut buff: Vec<u8>) -> StateMachine {
        if ::aces::Request::is_complete_request(&buff) {
            if let Ok(req) = ::aces::Request::parse_request(&buff) {
                let resp = device::response_for_request(&req);
                StateMachine::WriteResponse(resp)
            } else {
                StateMachine::ReadRequest(Vec::new())
            }
        } else {
            if let Some(mut chunk) = device.recv_timeout() {
                buff.append(&mut chunk);
                // clear the response
                device.set_response(&[]);
            }
            StateMachine::ReadRequest(buff)
        }
    }

    pub fn state_write_response(device: &mut device::Device, resp: Vec<u8>) -> StateMachine {
        device.set_response(&resp);

        StateMachine::ReadRequest(Vec::new())
    }

    use super::*;
}

mod device {
    pub fn setup_ble_device(name: &str) -> Device<'static> {
        log::info!("setting up BLE device ...");

        let adapter = BLEDevice::take();

        let advertising = adapter.get_advertising();
        let server = adapter.get_server();

        server.on_connect(|server, desc| {
            log::info!("client connected ({:?})", desc);

            server
                .update_conn_params(desc.conn_handle, 120, 120, 0, 60)
                .expect("failed to update conn params");
        });

        server.on_disconnect(move |_desc, reason| {
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

        Device {
            server,
            channel: channel::channel(tx, rx),
        }
    }

    pub fn response_for_request(req: &::aces::Request) -> Vec<u8> {
        match req {
            ::aces::Request::Clear => Vec::new(),
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
        }
    }

    pub struct Device<'s> {
        server: &'s mut BLEServer,
        channel: channel::Channel,
    }

    impl<'s> Device<'s> {
        pub fn is_connected(&self) -> bool {
            self.server.connected_count() > 0
        }

        /// Sets the response.
        pub fn set_response(&mut self, value: &[u8]) {
            self.channel.0.send(value);
        }

        /// Tries to receive the next value immediately.
        pub fn try_recv(&mut self) -> Option<Vec<u8>> {
            self.channel.1.recv_timeout(Duration::from_millis(10))
        }

        /// Tries to receive the next value within a reasonable time.
        pub fn recv_timeout(&mut self) -> Option<Vec<u8>> {
            self.channel.1.recv_timeout(Duration::from_millis(500))
        }
    }

    use crate::channel;
    use esp32_nimble::{utilities::BleUuid, BLEDevice, BLEServer, NimbleProperties};
    use std::time::Duration;
}

impl fmt::Debug for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateMachine::Disconnected => write!(f, "Disconnected"),
            StateMachine::Notify(req) => write!(f, "Notify({:?})", req),
            StateMachine::ReadRequest(req) => write!(f, "ReadRequest({:x?})", req),
            StateMachine::WriteResponse(resp) => write!(f, "WriteResponse({:x?})", resp),
        }
    }
}

use esp_idf_hal::task;
use esp_idf_sys as _;
use std::fmt;
