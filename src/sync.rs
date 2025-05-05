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

pub trait DerefExt<T> {
    unsafe fn get_ext(&self) -> &T;
    unsafe fn get_mut_ext(&self) -> &mut T;
}

impl<T> DerefExt<T> for UnsafeCell<T> {
    unsafe fn get_ext(&self) -> &T {
        unsafe {
            #[cfg(loom)]
            {
                self.get().with(|ptr| &*ptr)
            }
            #[cfg(not(loom))]
            {
                &*self.get()
            }
        }
    }

    unsafe fn get_mut_ext(&self) -> &mut T {
        unsafe {
            #[cfg(loom)]
            {
                self.get_mut().with(|ptr| &mut *ptr)
            }
            #[cfg(not(loom))]
            {
                &mut *self.get()
            }
        }
    }
}
