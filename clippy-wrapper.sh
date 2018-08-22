#!/bin/bash

exec rustup run nightly clippy-driver $@ --cfg procmacro2_semver_exempt
