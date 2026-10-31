@echo off
rem Build Auto Sway menjadi auto_sway.wasm (butuh Rust: https://rustup.rs)
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
if errorlevel 1 (
  echo Build gagal.
  pause
  exit /b 1
)
copy /Y target\wasm32-unknown-unknown\release\auto_sway.wasm auto_sway.wasm
echo Selesai: auto_sway.wasm
pause
