use crate::sync::{Condvar, Mutex};
use std::collections::VecDeque;

pub struct SafeChannel<T> {
    messages: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> SafeChannel<T> {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T) {
        self.messages.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    pub fn receive(&self) -> T {
        let mut m = self.messages.lock().unwrap();
        loop {
            if let Some(message) = m.pop_front() {
                return message;
            }
            m = self.item_ready.wait(m).unwrap();
        }
    }
}
