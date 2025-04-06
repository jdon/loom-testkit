use rust_atomics::locks::SpinLock;
use rust_atomics::{concurrent_test, sync};
use sync::Arc;
use sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst};
use sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};
use sync::spin_loop;
use sync::thread;
#[test]
fn test_concurrent_logic() {
    concurrent_test!({
        // Create an atomic counter shared between threads
        let v1 = Arc::new(AtomicUsize::new(0));
        let v2 = v1.clone();

        // Spawn a thread that updates the atomic value
        thread::spawn(move || {
            // Store 1 in the counter with SeqCst ordering
            // SeqCst ensures this operation is sequentially consistent
            // with all other SeqCst operations across all threads
            v1.store(1, SeqCst);
        });

        // This assertion may or may not pass depending on timing:
        // - If this executes before the spawned thread's store, it will be 0
        // - If this executes after the spawned thread's store, it will be 1
        // Loom will explore both possibilities during model checking
        assert_eq!(0, v2.load(SeqCst));
    });
}

#[test]
fn release_and_acquire_correct() {
    concurrent_test!({
        // Shared atomic variables between threads
        let data = Arc::new(AtomicU64::new(0));
        let ready = Arc::new(AtomicBool::new(false));

        // Spawn a worker thread
        thread::spawn({
            let data = data.clone();
            let ready = ready.clone();
            move || {
                // First update the data with Relaxed ordering
                // (ordering doesn't matter for this isolated store)
                data.store(42, Relaxed);

                // Signal that data is ready using Release ordering
                // Release ensures all previous memory operations are visible
                // to any thread that synchronizes with this operation
                ready.store(true, Release);
            }
        });

        // Wait until the ready flag is set, using Acquire ordering
        // Acquire creates a synchronization point with the Release store above
        // This forms a "release-acquire synchronization" that ensures memory visibility
        while !ready.load(Acquire) {
            spin_loop();
        }

        // After seeing ready=true with Acquire ordering, the Release-Acquire
        // synchronization guarantees that we will see the data=42 update
        assert_eq!(42, data.load(Relaxed));
    });
}

#[test]
fn release_and_acquire_incorrect() {
    concurrent_test!({
        // Shared atomic variables between threads
        let data = Arc::new(AtomicU64::new(0));
        let ready = Arc::new(AtomicBool::new(false));

        // Spawn a worker thread
        thread::spawn({
            let data = data.clone();
            let ready = ready.clone();
            move || {
                // First update the data with Relaxed ordering
                data.store(42, Relaxed);

                // Signal that data is ready using Release ordering
                ready.store(true, Release);
            }
        });

        // INCORRECT: Using Relaxed instead of Acquire creates a potential race condition
        // Without Acquire ordering, this load doesn't synchronize with the Release store
        // in the other thread, so there's no guarantee we'll see data=42 even after
        // seeing ready=true
        while !ready.load(Relaxed) {
            spin_loop();
        }

        // This assertion might fail in a real concurrent system or in Loom's model
        // checker, as there's no happens-before relationship established between
        // the data store and this load
        assert_eq!(42, data.load(Relaxed));
    });
}

#[test]
fn spin_lock() {
    concurrent_test!({
        // Create a shared lock around a tuple of (data, flag)
        let lock = Arc::new(SpinLock::new((false, false)));

        let lock_writer = lock.clone();
        let lock_reader = lock.clone();

        // Thread A: First writer thread
        let writer = thread::spawn(move || {
            // Acquire the lock - requires Acquire ordering internally
            let mut guard = lock_writer.lock();

            // Update the data value
            guard.0 = true;

            // Set the flag indicating data was modified
            guard.1 = true;

            // When guard is dropped here, the lock is released
            // The SpinLock implementation must use Release ordering
            // to make these writes visible to the next thread acquiring the lock
        });

        // Thread B: Reader thread
        let reader = thread::spawn(move || {
            // Acquire the lock - requires Acquire ordering internally
            // to synchronize with the Release in the writer thread
            let guard = lock_reader.lock();

            // Check if the flag is set
            if guard.1 {
                // If the flag is set, the data must also be set
                // This will ONLY be guaranteed if the SpinLock uses
                // proper Acquire/Release semantics in its implementation
                assert!(guard.0)
            }
        });

        // Wait for both threads to complete
        writer.join().unwrap();
        reader.join().unwrap();
    });
}
