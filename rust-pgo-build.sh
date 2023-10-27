#!/usr/bin/env bash

set -ex

# install prerequisistes
rustup override set nightly
rustup component add llvm-tools
cargo profdata -- --help >/dev/null 2>&1 || cargo install cargo-binutils

find . -name "*.profraw" -delete
export RUSTFLAGS="-C profile-generate"
cargo run --release
find . -name "*.profraw"
cargo profdata -- merge -o merged.profdata *.profraw
find . -name "*.profraw" -delete
export RUSTFLAGS="-C profile-use=$(realpath ./merged.profdata)"
cargo clean
cargo build --release

rustup override set stable
