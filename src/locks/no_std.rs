use atomic_wait::{wait, wake_one};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicU32,
};

pub struct Mutex<T> {
    /// 0: unlocked
    /// 1: locked, no other thread(s) waiting
    /// 2: locked, other thread(s) waiting
    state: AtomicU32,
    value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Mutex<T> {
        Mutex {
            state: AtomicU32::new(0), // unlocked
            value: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        if self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
            // The lock was already locked. :(
            lock_contended(&self.state);
        }
        MutexGuard { mutex: self }
    }
}

#[cold]
#[inline]
fn lock_contended(state: &AtomicU32) {
    let mut spin_count = 0;

    // Spin initially to avoid the 'wait' syscall
    // We only spin while there are no other waiters
    // Relaxed memory ordering is fine as we don't require a happens before relationship here
    while state.load(Relaxed) == 1 && spin_count < 100 {
        spin_count += 1;
        std::hint::spin_loop();
    }

    // If after spinning the lock is unlocked, we attempt to lock it
    if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
        return;
    }

    // Still locked, so swap to 2 as we are the other thread waiting for the lock
    while state.swap(2, Acquire) != 0 {
        wait(state, 2);
    }
}
unsafe impl<T> Sync for Mutex<T> where T: Send {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Unlock the mutex by setting the value to 0
        // If we get '2', then we know another thread was waiting for th lock, so wake it
        if self.mutex.state.swap(0, Release) == 2 {
            // Wait one thread
            // The waking doesn't form part of the safeness of this mutex
            // It's purely an optimisation
            wake_one(&self.mutex.state);
        }
    }
}

