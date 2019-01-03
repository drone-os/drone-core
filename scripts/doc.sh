#!/bin/bash

export RUSTC_WRAPPER=$(dirname $0)/_rustc_wrapper.sh

set -x
cargo doc --all
