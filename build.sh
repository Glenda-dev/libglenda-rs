#!/bin/sh
set -e

for arg in "$@"; do
  if [ "$arg" = "--release" ]; then
    MODE="release"
  fi
done

# Build the project
cargo build --target riscv64gc-unknown-none-elf $@