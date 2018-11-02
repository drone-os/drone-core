#!/bin/bash

set -x

cargo readme -o README.md
cargo readme -r ctypes -t ../README.tpl -o README.md
