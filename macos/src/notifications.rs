pub struct Notifications {
    notif: Arc<Mutex<VecDeque<Vec<u8>>>>,
    cv: Arc<Condvar>,
}

impl Notifications {
    pub async fn subscribe(
        peripheral: &Peripheral,
        characteristic: Characteristic,
    ) -> Result<Notifications, Box<dyn std::error::Error>> {
        let notif = Arc::new(Mutex::new(VecDeque::new()));
        let cv = Arc::new(Condvar::new());

        peripheral.subscribe(&characteristic).await?;
        let mut notifs = peripheral.notifications().await?;

        let notif0 = Arc::clone(&notif);
        let cv0 = Arc::clone(&cv);
        tokio::task::spawn(async move {
            loop {
                if let Some(notif) = notifs.next().await {
                    log::trace!("received notification item from stream");
                    let mut lock = notif0.lock().unwrap();
                    lock.push_back(notif.value);
                    cv0.notify_one();
                } else {
                    log::trace!("did not notification item from stream");
                }
                tokio::task::yield_now().await;
            }
        });

        Ok(Notifications { notif, cv })
    }
}

impl aces::NotificationsReceiver for Notifications {
    fn next(&mut self) -> Vec<u8> {
        log::debug!("awaiting next notification");

        let mut locked = self.notif.lock().unwrap();
        // protect agains spurious wake-ups
        while locked.is_empty() {
            locked = self.cv.wait(locked).unwrap();
        }

        let val = locked.pop_front().unwrap();
        self.cv.notify_one();
        val
    }
}

use btleplug::{
    api::{Characteristic, Peripheral as _},
    platform::Peripheral,
};
use futures::StreamExt;
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};
