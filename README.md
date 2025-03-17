# Loom Test Kit

A practical demonstration of testing concurrent Rust code using [Loom](https://github.com/tokio-rs/loom), a model checker that systematically explores thread interleavings to find concurrency bugs.

## Why Use Loom?

Traditional testing approaches can't reliably detect concurrency bugs because:
- Bugs may appear in only a tiny fraction of possible thread interleavings
- Some issues won't appear even after millions of random test runs
- Problems with memory ordering may only manifest on specific hardware

Loom provides systematic exploration rather than random testing, making it much more effective for concurrent code.

## Workflow: Finding Concurrency Bugs with Loom

Follow these steps to find and diagnose concurrency bugs:

1. **Run standard tests** - They'll likely pass despite having bugs:
   ```bash
   ./run_normal_tests.sh
   ```

2. **Run with Loom** - This will find concurrency issues:
   ```bash
   ./run_loom.sh
   ```

3. **Create a checkpoint** for the failing test:
   ```bash
   ./checkpoint_loom.sh test_concurrent_logic
   ```

4. **Get a detailed trace** to diagnose the issue:
   ```bash
   ./trace_loom.sh test_concurrent_logic
   ```

5. **Examine the trace output** to understand the specific thread interleaving that caused the failure.

6. **Clear artifacts** after you're done analyzing:
   ```bash
   ./clear_loom_artifacts.sh
   ```

## Testing Scripts Explained

### 1. `run_normal_tests.sh`

Runs tests normally without Loom, using standard Rust threading:

```bash
cargo test --release
```

Most concurrency bugs won't be detected in this mode, as they only occur in specific thread interleavings.

### 2. `run_loom.sh`

Runs all tests with Loom enabled:

```bash
RUSTFLAGS="--cfg loom" cargo test --release
```

This activates Loom's model checking, which systematically explores thread interleavings to find bugs.

### 3. `checkpoint_loom.sh <test_name>`

Creates a checkpoint file for a failing test:

```bash
RUSTFLAGS="--cfg loom" \
LOOM_CHECKPOINT_INTERVAL=1 \
LOOM_CHECKPOINT_FILE="loom_test_artifacts/${TEST_NAME}.json" \
cargo test --release "$TEST_NAME"
```

Checkpoints are saved to `loom_test_artifacts/` and help pinpoint where a failure occurs.

### 4. `trace_loom.sh <test_name>`

Provides detailed trace logs for a failing test:

```bash
RUSTFLAGS="--cfg loom" \
LOOM_LOG=trace \
LOOM_LOCATION=1 \
LOOM_CHECKPOINT_INTERVAL=1 \
LOOM_CHECKPOINT_FILE="loom_test_artifacts/${TEST_NAME}.json" \
cargo test --release "$TEST_NAME"
```

Traces show the exact execution path that led to the failure, including thread scheduling decisions.

### 5. `clear_loom_artifacts.sh`

Cleans up Loom test artifacts:

```bash
./clear_loom_artifacts.sh
```

Removes all files in the `loom_test_artifacts/` directory or creates the directory if it doesn't exist.

## Project Structure

### `concurrent_test` Macro

For writing tests that work in both normal and Loom modes:

```rust
#[test]
fn my_test() {
    concurrent_test!({
        // Test code here
        // Will run directly in normal mode
        // Will run inside loom::model in Loom mode
    })
}
```

### Configuration Setup

- **sync.rs module**: Abstracts synchronization primitives between std and Loom
- **loom config flag**: Enables conditional compilation with `#[cfg(loom)]`
- **Cargo.toml**: Includes Loom as a conditional dependency:
  ```toml
  [target.'cfg(loom)'.dependencies]
  loom = { version = "0.7", features = ["checkpoint"] }
  ```
