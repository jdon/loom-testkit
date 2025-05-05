use std::mem::MaybeUninit;

use crate::sync::*;
pub struct UnsafeOneShotChannel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: atomic::AtomicBool,
}

unsafe impl<T> Sync for UnsafeOneShotChannel<T> where T: Send {}

impl<T> UnsafeOneShotChannel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: atomic::AtomicBool::new(false),
        }
    }

    pub unsafe fn send(&self, message: T) {
        let message_holder = unsafe { self.message.get_mut_ext() };
        message_holder.write(message);
        self.ready.store(true, atomic::Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(atomic::Ordering::Acquire)
    }

    pub unsafe fn receive(&self) -> T {
        unsafe {
            let message_holder = self.message.get_mut_ext();

            message_holder.assume_init_read()
        }
    }
}
