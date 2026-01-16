$ErrorActionPreference = "Stop"

$bin = "code-rag"
$dist = Join-Path $PSScriptRoot "..\release\windows"
# We don't strictly need a release folder for debug, but to keep it consistent 
# or just let it live in target/debug.
# The user asked for a debug build. Standard is just `cargo build`.
# I will output it to target\debug and maybe copy to a release\debug if useful?
# The plan said: "runs cargo build --bin code-rag".
# The user said: "create a debug build in powershell".

Write-Host "==> Building Windows MSVC binary (DEBUG) locally..." -ForegroundColor Cyan

# Ensure MSVC toolchain
# rustup default stable-x86_64-pc-windows-msvc | Out-Null

cargo build --bin $bin

$exe = "target\debug\$bin.exe"
if (!(Test-Path $exe)) {
    throw "Expected output not found: $exe"
}

Write-Host "Done. Windows binary (DEBUG): $exe" -ForegroundColor Green
