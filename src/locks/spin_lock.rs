use std::ops::{Deref, DerefMut};

use crate::sync::UnsafeCell;
use crate::sync::atomic::AtomicBool;
use crate::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use crate::sync::spin_loop;

pub struct SpinLock<T> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub fn new(inner: T) -> SpinLock<T> {
        SpinLock {
            lock: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        loop {
            match self
                .lock
                .compare_exchange_weak(false, true, Acquire, Relaxed)
            {
                Ok(_) => return Guard { inner: self },
                Err(_) => spin_loop(),
            }
        }
    }
}

pub struct Guard<'a, T> {
    inner: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.inner.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.inner.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.inner.lock.store(false, Release);
    }
}
