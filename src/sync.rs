#[cfg(loom)]
pub use loom::cell::*;
#[cfg(loom)]
pub use loom::hint::spin_loop;
#[cfg(loom)]
pub use loom::sync::*;
#[cfg(loom)]
pub use loom::thread;

#[cfg(not(loom))]
pub use std::cell::*;
#[cfg(not(loom))]
pub use std::hint::spin_loop;
#[cfg(not(loom))]
pub use std::sync::*;
#[cfg(not(loom))]
pub use std::thread;
