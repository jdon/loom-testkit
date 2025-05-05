mod no_std;
mod not_std_rwlock;
mod spin_lock;

pub use no_std::{Mutex, MutexGuard};
pub use not_std_rwlock::{ReadGuard, RwLock, WriteGuard};
pub use spin_lock::{Guard, SpinLock};
