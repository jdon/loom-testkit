#!/bin/bash

# Check if the artifacts directory exists
if [ -d "loom_test_artifacts" ]; then
    echo "Clearing Loom test artifacts..."
    rm -rf loom_test_artifacts/*
    echo "Done."
else
    echo "No Loom artifacts directory found. Creating empty directory..."
    mkdir -p loom_test_artifacts
    echo "Done."
fi
