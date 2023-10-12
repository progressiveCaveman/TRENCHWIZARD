#!/bin/sh

# RUST_BACKTRACE=1 cargo run
sudo RUST_BACKTRACE=1 cargo run --release

# for profiling
# sudo cargo flamegraph