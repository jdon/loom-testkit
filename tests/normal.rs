use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::{sync::Arc, thread};

#[test]
fn no_std_lock() {
    use rust_atomics::locks::Mutex;
    // Create a shared lock around a tuple of (data, flag)
    let lock = Arc::new(Mutex::new((false, false)));

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
        // The implementation must use Release ordering
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
            // This will ONLY be guaranteed if the lock uses
            // proper Acquire/Release semantics in its implementation
            assert!(guard.0)
        }
    });

    // Wait for both threads to complete
    writer.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn no_std_rwlock() {
    use rust_atomics::locks::RwLock;

    // Create a shared lock around a tuple of (data, flag)
    let lock = Arc::new(RwLock::new((false, false)));

    let lock_writer = lock.clone();
    let lock_reader = lock.clone();

    // Thread A: Writer thread
    let writer = thread::spawn(move || {
        // Acquire the write lock - requires Acquire ordering internally
        let mut guard = lock_writer.write();

        // Update the data value
        guard.0 = true;

        // Set the flag indicating data was modified
        guard.1 = true;

        // When guard is dropped here, the write lock is released
        // The implementation must use Release ordering
        // to make these writes visible to the next thread acquiring the lock
    });

    // Thread B: Reader thread
    let reader = thread::spawn(move || {
        // Acquire the read lock - requires Acquire ordering internally
        // to synchronize with the Release in the writer thread
        let guard = lock_reader.read();

        // Check if the flag is set
        if guard.1 {
            // If the flag is set, the data must also be set
            // This will ONLY be guaranteed if the lock uses
            // proper Acquire/Release semantics in its implementation
            assert!(guard.0)
        }
    });

    // Wait for both threads to complete
    writer.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn rwlock_multiple_readers() {
    use rust_atomics::locks::RwLock;
    use std::sync::atomic::{AtomicBool, Ordering};

    const NUM_READERS: usize = 5;
    let lock = Arc::new(RwLock::new(42));
    let started = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(NUM_READERS);

    // Spawn several reader threads
    for _ in 0..NUM_READERS {
        let lock = lock.clone();
        let started = started.clone();

        let handle = thread::spawn(move || {
            // Wait until all threads are ready
            while !started.load(Ordering::Relaxed) {
                thread::yield_now();
            }

            // All threads should be able to read concurrently
            let guard = lock.read();
            assert_eq!(*guard, 42);

            // Sleep to increase likelihood of concurrent reads
            thread::sleep(Duration::from_millis(10));
        });

        handles.push(handle);
    }

    // Signal all threads to start
    started.store(true, Ordering::Relaxed);

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn rwlock_writer_exclusion() {
    use rust_atomics::locks::RwLock;

    let lock = Arc::new(RwLock::new(0));
    let counter = Arc::new(AtomicUsize::new(0));

    // First, create a write lock
    let lock_writer = lock.clone();
    let counter_writer = counter.clone();

    let writer = thread::spawn(move || {
        // Acquire the write lock
        let mut guard = lock_writer.write();

        // Modify the value
        *guard = 100;

        // Hold the lock for a while to ensure the reader threads have to wait
        counter_writer.store(1, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(50));
        counter_writer.store(2, Ordering::SeqCst);

        // Write again before releasing
        *guard = 200;
    });

    // Give the writer a chance to acquire the lock
    thread::sleep(Duration::from_millis(10));

    // Now try to read - this should block until the writer is done
    let lock_reader = lock.clone();
    let counter_reader = counter.clone();

    let reader = thread::spawn(move || {
        // Check the counter value before acquiring the lock
        let before = counter_reader.load(Ordering::SeqCst);

        // Try to acquire the read lock (this should block)
        let guard = lock_reader.read();

        // Check the counter value after acquiring the lock
        let after = counter_reader.load(Ordering::SeqCst);

        // The value should be 200 (the final write from the writer thread)
        assert_eq!(*guard, 200);

        // We should only get access after the writer thread stored 2
        assert_eq!(after, 2);

        // If we got here before the writer stored 1, the lock isn't working
        assert!(
            before >= 1,
            "Reader acquired lock before writer or concurrently"
        );
    });

    writer.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn rwlock_reader_to_writer_upgrade() {
    use rust_atomics::locks::RwLock;

    let lock = Arc::new(RwLock::new(vec![1, 2, 3]));

    // First get a read lock
    let read_guard = lock.read();

    // Verify we can read the data
    assert_eq!(*read_guard, vec![1, 2, 3]);

    // Drop the read guard to be able to acquire a write guard
    drop(read_guard);

    // Now get a write lock
    let mut write_guard = lock.write();

    // Modify the data
    write_guard.push(4);

    // Verify modification
    assert_eq!(*write_guard, vec![1, 2, 3, 4]);
}

#[test]
fn rwlock_writer_to_reader_downgrade() {
    use rust_atomics::locks::RwLock;

    let lock = Arc::new(RwLock::new(10));

    // First get a write lock
    let mut write_guard = lock.write();

    // Modify the data
    *write_guard = 20;

    // Drop the write guard to be able to acquire a read guard
    drop(write_guard);

    // Now get a read lock
    let read_guard = lock.read();

    // Verify we can read the modified data
    assert_eq!(*read_guard, 20);
}

#[test]
fn rwlock_stress_test() {
    use rust_atomics::locks::RwLock;

    const NUM_THREADS: usize = 10;
    const OPS_PER_THREAD: usize = 100;

    let lock = Arc::new(RwLock::new(0));
    let mut handles = Vec::with_capacity(NUM_THREADS);

    // Create multiple threads that alternate between reading and writing
    for id in 0..NUM_THREADS {
        let lock = lock.clone();

        let handle = thread::spawn(move || {
            for i in 0..OPS_PER_THREAD {
                if (id + i) % 5 == 0 {
                    // Writer operation
                    let mut guard = lock.write();
                    *guard += 1;
                } else {
                    // Reader operation
                    let guard = lock.read();
                    assert!(*guard >= 0); // Simple validation
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Final check - read the final value
    let final_value = *lock.read();

    // The final value should be equal to the total number of write operations
    // NUM_THREADS * OPS_PER_THREAD / 5 (since we do a write every 5 operations)
    let expected_writes = NUM_THREADS * OPS_PER_THREAD / 5;
    assert_eq!(final_value, expected_writes);
}

#[test]
fn rwlock_read_timeout_consistency() {
    use rust_atomics::locks::RwLock;
    use std::sync::mpsc::channel;

    let lock = Arc::new(RwLock::new(0));

    // Set up a channel for communication
    let (sender, receiver) = channel();

    // Spawn a writer thread that holds the lock for a while
    let writer_lock = lock.clone();
    let writer = thread::spawn(move || {
        // Acquire the write lock
        let mut guard = writer_lock.write();

        // Signal that we have the lock
        sender.send(()).unwrap();

        // Hold the lock for a while
        thread::sleep(Duration::from_millis(100));

        // Update the value before releasing
        *guard = 42;
    });

    // Wait for the writer to signal it has the lock
    receiver.recv().unwrap();

    // Spawn multiple reader threads that try to read
    let reader_lock = lock.clone();
    let reader1 = thread::spawn(move || {
        // This should block until the writer releases
        let start = Instant::now();
        let guard = reader_lock.read();
        let elapsed = start.elapsed();

        // Verify the value is what the writer set
        assert_eq!(*guard, 42);

        // Verify we blocked long enough
        assert!(elapsed.as_millis() >= 50, "Reader didn't wait for writer");
    });

    // Wait for all threads
    writer.join().unwrap();
    reader1.join().unwrap();
}
