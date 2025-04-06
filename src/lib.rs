pub mod locks;
pub mod sync;

#[macro_export]
macro_rules! concurrent_test {
    ($body:expr) => {
        #[cfg(not(loom))]
        {
            $body
        }

        #[cfg(loom)]
        {
            loom::model(|| $body)
        }
    };
}
