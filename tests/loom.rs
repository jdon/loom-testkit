use rust_atomics::{concurrent_test, sync};
use sync::Arc;
use sync::atomic::AtomicUsize;
use sync::atomic::Ordering::SeqCst;
use sync::thread;

#[test]
fn test_concurrent_logic() {
    concurrent_test!({
        let v1 = Arc::new(AtomicUsize::new(0));
        let v2 = v1.clone();

        thread::spawn(move || {
            v1.store(1, SeqCst);
        });
        assert_eq!(0, v2.load(SeqCst));
    });
}
