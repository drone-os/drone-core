#!/bin/bash

exec rustup run \
  $(cat $(dirname "$0")/../rust-toolchain) \
  clippy-driver "$@" \
  --cfg procmacro2_semver_exempt
