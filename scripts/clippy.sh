#!/bin/bash

export RUSTC_WRAPPER=$(dirname $0)/clippy-wrapper.sh
set -x

cargo check --all
