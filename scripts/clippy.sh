#!/bin/bash

export RUSTC_WRAPPER=$(dirname $0)/_clippy_wrapper.sh

set -x
cargo check --all
