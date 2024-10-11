pub struct Notifications {
    state: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
}

impl Notifications {
    pub async fn subscribe(
        peripheral: &Peripheral,
        characteristic: Characteristic,
    ) -> Result<Notifications, Box<dyn std::error::Error>> {
        let state = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));

        peripheral.subscribe(&characteristic).await?;
        let mut notifs = peripheral.notifications().await?;

        let state0 = Arc::clone(&state);
        tokio::task::spawn(async move {
            loop {
                if let Some(notif) = notifs.next().await {
                    log::trace!("received notification item from stream");
                    let mut lock = state0.0.lock().unwrap();
                    lock.push_back(notif.value);
                    state0.1.notify_one();
                } else {
                    log::trace!("did not receive notification item from stream");
                }
                tokio::task::yield_now().await;
            }
        });

        Ok(Notifications { state })
    }
}

impl aces::NotificationsReceiver for Notifications {
    fn next(&mut self) -> Vec<u8> {
        log::debug!("awaiting next notification");

        let mut locked = self.state.0.lock().unwrap();
        // protect agains spurious wake-ups
        while locked.is_empty() {
            locked = self.state.1.wait(locked).unwrap();
        }

        let val = locked.pop_front().unwrap();
        self.state.1.notify_one();
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
