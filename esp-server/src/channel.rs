pub struct Sender(Arc<Mutex<BLECharacteristic>>);

pub struct Receiver {
    state: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
}

pub type Channel = (Sender, Receiver);

pub fn channel(tx: Arc<Mutex<BLECharacteristic>>, rx: Arc<Mutex<BLECharacteristic>>) -> Channel {
    (Sender(tx), Receiver::new(rx))
}

impl Sender {
    pub fn send(&mut self, value: &[u8]) {
        let mut char = self.0.lock();
        char.set_value(value).notify();
    }
}

impl Receiver {
    fn new(chr: Arc<Mutex<BLECharacteristic>>) -> Self {
        let state = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));

        let state0 = Arc::clone(&state);
        chr.lock().on_write(move |args| {
            let value = args.recv_data.to_vec();
            log::debug!("got write {:x?}", value);
            let mut lock = state0.0.lock();
            lock.push_back(value);
            state0.1.notify_one();
        });

        Receiver { state }
    }

    pub fn recv_timeout(&mut self, duration: Duration) -> Option<Vec<u8>> {
        let mut locked = self.state.0.lock();

        if locked.is_empty() {
            let timeout;
            (locked, timeout) = self.state.1.wait_timeout(locked, duration);

            if timeout {
                return None;
            }
        }

        let val = locked.pop_front();
        self.state.1.notify_one();
        val
    }
}

use esp32_nimble::{
    utilities::mutex::{Condvar, Mutex},
    BLECharacteristic,
};
use std::{collections::VecDeque, sync::Arc, time::Duration};
