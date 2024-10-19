pub struct Notifications {
    rx: sync::Receiver<Vec<u8>>,
}

impl Notifications {
    pub async fn subscribe(
        peripheral: &Peripheral,
        characteristic: Characteristic,
    ) -> Result<Notifications, Box<dyn std::error::Error>> {
        peripheral.subscribe(&characteristic).await?;

        let mut notifs = peripheral.notifications().await?;
        let (rx, tx) = sync::channel();

        tokio::task::spawn(async move {
            loop {
                if let Some(notif) = notifs.next().await {
                    log::trace!("received notification item from stream");
                    tx.send(notif.value);
                } else {
                    log::trace!("did not receive notification item from stream");
                }
                tokio::task::yield_now().await;
            }
        });

        Ok(Notifications { rx })
    }

    pub fn clear(&mut self) {
        self.rx.clear();
    }
}

impl aces::NotificationsReceiver for Notifications {
    fn next(&mut self) -> Vec<u8> {
        log::debug!("awaiting next notification");
        self.rx.recv()
    }
}

mod sync {
    /// Creates a new channel.
    pub fn channel<T>() -> (Receiver<T>, Sender<T>) {
        let inner = Inner(Arc::new(((Mutex::new(VecDeque::new())), Condvar::new())));
        (Receiver(inner.clone()), Sender(inner))
    }

    pub struct Sender<T>(Inner<T>);

    pub struct Receiver<T>(Inner<T>);

    impl<T> Sender<T> {
        pub fn send(&self, val: T) {
            let mut locked = self.0 .0 .0.lock().unwrap();
            locked.push_back(val);
            self.0 .0 .1.notify_one();
        }
    }

    impl<T> Receiver<T> {
        pub fn recv(&self) -> T {
            let mut locked = self.0 .0 .0.lock().unwrap();

            // protect agains spurious wake-ups
            while locked.is_empty() {
                locked = self.0 .0 .1.wait(locked).unwrap();
            }

            let val = locked.pop_front().unwrap();
            self.0 .0 .1.notify_one();
            val
        }

        pub fn clear(&self) {
            let mut locked = self.0 .0 .0.lock().unwrap();
            locked.clear();
            self.0 .0 .1.notify_one();
        }
    }

    struct Inner<T>(Arc<(Mutex<VecDeque<T>>, Condvar)>);

    impl<T> Clone for Inner<T> {
        fn clone(&self) -> Self {
            Inner(self.0.clone())
        }
    }

    use std::{
        collections::VecDeque,
        sync::{Arc, Condvar, Mutex},
    };
}

use btleplug::{
    api::{Characteristic, Peripheral as _},
    platform::Peripheral,
};
use futures::StreamExt;
