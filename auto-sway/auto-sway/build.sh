#!/bin/sh
# Build Auto Sway menjadi auto_sway.wasm (butuh Rust: https://rustup.rs)
set -e
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/auto_sway.wasm auto_sway.wasm
echo "Selesai: auto_sway.wasm"
