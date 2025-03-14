# Rust Atomics with Loom Testing

This repository contains code for testing concurrent Rust code using [Loom](https://github.com/tokio-rs/loom), a model checker for concurrent Rust code.

## Project Structure

- **sync.rs module**: Abstracts synchronization primitives to use either standard library (`std::sync`, `std::thread`) in normal builds or Loom's primitives (`loom::sync`, `loom::thread`) during testing.

- **loom config flag**: Enables conditional compilation:
  ```rust
  #[cfg(loom)]     // Used with Loom testing
  #[cfg(not(loom))] // Used in normal builds
  ```
  When running Loom tests, we set `--cfg loom` via `RUSTFLAGS` to activate Loom's implementation.

## The `concurrent_test` Macro

The project includes a `concurrent_test` macro that simplifies writing tests that work both with and without Loom.

This macro allows you to:
- Write a single test that works in both normal and Loom testing modes
- Automatically wrap the test code in `loom::model` when Loom is enabled
- Execute the same code directly when running normal tests

Example usage:

```rust
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
```

With this approach, you write your test once, and it behaves differently depending on whether you're running normal tests or Loom tests.

## Why Use Loom?

Concurrent code testing is fundamentally challenging:

- Bugs may occur in only a tiny fraction of possible thread interleavings
- Some issues won't appear even after millions of test runs
- Problems with relaxed memory ordering may only manifest on specific hardware

Loom addresses these challenges by:

- Deterministically exploring all valid execution permutations
- Simulating Rust's memory model and the OS scheduler
- Verifying correctness across all possible executions, not just "most of the time"

Traditional approaches (running tests in loops or under system load) are unreliable for finding these issues:

```bash
./run_normal_tests.sh  # Tests pass in normal mode
./run_loom.sh          # Tests fail under Loom's model checker
```

## Testing Scripts

### Normal Testing
```bash
./run_normal_tests.sh
```

### Loom Testing
1. **Run all Loom tests**:
   ```bash
   ./run_loom.sh
   ```

2. **Checkpoint a failing test**:
   ```bash
   ./checkpoint_loom.sh test_name
   ```
   Creates a checkpoint at `loom_test_artifacts/test_name.json`

3. **Trace a failing test**:
   ```bash
   ./trace_loom.sh test_name
   ```
   Provides detailed trace logs to diagnose the failure.

## Workflow Example
```bash
# Run normal and Loom tests
./run_normal_tests.sh
./run_loom.sh

# If "concurrent_bug" fails, create checkpoint and trace
./checkpoint_loom.sh concurrent_bug
./trace_loom.sh concurrent_bug
```

Trace information and checkpoint files are stored in the `loom_test_artifacts` directory.