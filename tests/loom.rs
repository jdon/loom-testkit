use rust_atomics::{concurrent_test, sync};
use sync::Arc;
use sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst};
use sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};
use sync::spin_loop;
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

#[test]
fn release_and_acquire_correct() {
    concurrent_test!({
        let data = Arc::new(AtomicU64::new(0));
        let ready = Arc::new(AtomicBool::new(false));
        thread::spawn({
            let data = data.clone();
            let ready = ready.clone();
            move || {
                data.store(42, Relaxed);
                ready.store(true, Release);
            }
        });

        while !ready.load(Acquire) {
            spin_loop();
        }
        assert_eq!(42, data.load(Relaxed));
    });
}

#[test]
fn release_and_acquire_incorrect() {
    concurrent_test!({
        let data = Arc::new(AtomicU64::new(0));
        let ready = Arc::new(AtomicBool::new(false));
        thread::spawn({
            let data = data.clone();
            let ready = ready.clone();
            move || {
                data.store(42, Relaxed);
                ready.store(true, Release);
            }
        });

        // This should be Acquire instead of Relaxed
        while !ready.load(Relaxed) {
            spin_loop();
        }
        assert_eq!(42, data.load(Relaxed));
    });
}
