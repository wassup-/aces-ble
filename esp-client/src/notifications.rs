pub struct Notifications {
    state: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
}

impl Notifications {
    pub async fn subscribe(characteristic: &mut BLERemoteCharacteristic) -> Self {
        let state = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));

        let state0 = Arc::clone(&state);
        characteristic
            .on_notify(move |value| {
                log::debug!("got notification");
                let mut lock = state0.0.lock();
                lock.push_back(value.to_vec());
                state0.1.notify_one();
            })
            .subscribe_notify(false)
            .await
            .unwrap();

        Notifications { state }
    }

    pub fn clear(&mut self) {
        let mut lock = self.state.0.lock();
        lock.clear();
        self.state.1.notify_one();
    }
}

impl aces::NotificationsReceiver for Notifications {
    fn next(&mut self) -> Vec<u8> {
        log::debug!("awaiting next notification");

        let mut locked = self.state.0.lock();
        // protect agains spurious wake-ups
        while locked.is_empty() {
            locked = self.state.1.wait(locked);
        }

        let val = locked.pop_front().unwrap();
        self.state.1.notify_one();
        val
    }
}

use esp32_nimble::{
    utilities::mutex::{Condvar, Mutex},
    BLERemoteCharacteristic,
};
use std::{collections::VecDeque, sync::Arc};
