pub mod sync;
pub mod locks;

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
