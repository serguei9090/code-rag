$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path "release\linux" | Out-Null
New-Item -ItemType Directory -Force -Path "release\windows" | Out-Null

Write-Host "==> Building Linux binary (x86_64-unknown-linux-gnu)..." -ForegroundColor Cyan
docker build `
  -f Dockerfile.build `
  --build-arg TARGET=x86_64-unknown-linux-gnu `
  --output "type=local,dest=release\linux" `
  .

Write-Host "==> Building Windows binary (x86_64-pc-windows-gnu)..." -ForegroundColor Cyan
docker build `
  -f Dockerfile.build `
  --build-arg TARGET=x86_64-pc-windows-gnu `
  --output "type=local,dest=release\windows" `
  .

Write-Host ""
Write-Host "Done." -ForegroundColor Green
Write-Host "Linux binary  : release\linux\code-rag"
Write-Host "Windows binary: release\windows\code-rag.exe"
