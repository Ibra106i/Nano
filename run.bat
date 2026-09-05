@echo off
echo Building Nano...
cd /d "%~dp0"
cargo build --release 2>nul
if %errorlevel% neq 0 (
    echo Build failed! Running debug build instead...
    cargo run
) else (
    echo Starting Nano...
    start "" "target\release\rust-editor.exe"
)
