$ErrorActionPreference = "Stop"

$bin = "code-rag"
$dist = Join-Path $PSScriptRoot "..\release\windows"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "==> Building Windows MSVC binary locally..." -ForegroundColor Cyan

# Ensure MSVC toolchain
# rustup default stable-x86_64-pc-windows-msvc | Out-Null

# Clean is optional; useful when switching targets or fixing native deps
# cargo clean

cargo build --release --bin $bin

$exe = "target\release\$bin.exe"
if (!(Test-Path $exe)) {
  throw "Expected output not found: $exe"
}

Copy-Item $exe (Join-Path $dist "$bin.exe") -Force

Write-Host "Done. Windows binary: release\windows\$bin.exe" -ForegroundColor Green
