pub struct Notifications {
    notif: Arc<Mutex<VecDeque<Vec<u8>>>>,
    cv: Arc<Condvar>,
}

impl Notifications {
    pub async fn subscribe(characteristic: &mut BLERemoteCharacteristic) -> Self {
        let notif = Arc::new(Mutex::new(VecDeque::new()));
        let cv = Arc::new(Condvar::new());

        let notif0 = Arc::clone(&notif);
        let cv0 = Arc::clone(&cv);
        characteristic
            .on_notify(move |value| {
                log::debug!("got notification");
                let mut lock = notif0.lock();
                lock.push_back(value.to_vec());
                cv0.notify_one();
            })
            .subscribe_notify(false)
            .await
            .unwrap();

        Notifications { notif, cv }
    }
}

impl aces::Notifications for Notifications {
    fn next(&mut self) -> Vec<u8> {
        log::debug!("awaiting next notification");

        let mut lock = self.notif.lock();
        // protect agains spurious wake-ups
        while lock.is_empty() {
            lock = self.cv.wait(lock);
        }

        let val = lock.pop_front().unwrap();
        self.cv.notify_one();
        val
    }
}

use esp32_nimble::{
    utilities::mutex::{Condvar, Mutex},
    BLERemoteCharacteristic,
};
use std::{collections::VecDeque, sync::Arc};
