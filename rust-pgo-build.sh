#!/usr/bin/env bash

set -ex

# install prerequisistes
rustup override set nightly
rustup component add llvm-tools-preview
cargo pgo -- --help >/dev/null 2>&1 || cargo install cargo-pgo

cargo pgo run -- --features pgo
cargo pgo optimize -- build --features pgo

rustup override set stable
