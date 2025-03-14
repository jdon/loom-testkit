#!/bin/bash

# Check if a test name was provided
if [ $# -eq 0 ]; then
    echo "Error: Test name is required"
    echo "Usage: $0 <test_name>"
    exit 1
fi

# Get the test name from the first argument
TEST_NAME=$1

# Create the artifacts directory if it doesn't exist
mkdir -p loom_test_artifacts

# Use the test name and artifacts directory for the checkpoint file
RUSTFLAGS="--cfg loom" \
LOOM_LOG=trace \
LOOM_LOCATION=1 \
LOOM_CHECKPOINT_INTERVAL=1 \
LOOM_CHECKPOINT_FILE="loom_test_artifacts/${TEST_NAME}.json" \
cargo test --release "$TEST_NAME"