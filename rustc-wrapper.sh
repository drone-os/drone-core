#!/bin/bash

input=($@)
output=()
rustc=0
for item in "${input[@]}"; do
  if [ "rustc" = "${item}" ]; then
    rustc=1
  else
    output+=("${item}")
  fi
done
if [ "$rustc" = "1" ]; then
  output=("rustc" "${output[@]}")
fi

exec "${output[@]}" --cfg procmacro2_semver_exempt
