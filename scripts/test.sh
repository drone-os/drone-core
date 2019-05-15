#!/bin/bash

set -x

cargo test --all --exclude drone-core
cargo test --features "std" -p drone-core
