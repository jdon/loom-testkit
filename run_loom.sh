#!/bin/bash

RUSTFLAGS="--cfg loom" \
cargo test --release