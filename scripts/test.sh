#!/bin/bash

export RUSTC_WRAPPER=$(dirname $0)/rustc-wrapper.sh
set -x

cargo test --all --exclude drone-core
cargo test --features "std" -p drone-core
