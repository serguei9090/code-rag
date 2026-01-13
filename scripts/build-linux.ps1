$ErrorActionPreference = "Stop"

$bin = "code-rag"
$dist = Join-Path $PSScriptRoot "..\release\linux"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "==> Building Linux binary using Docker..." -ForegroundColor Cyan

docker build `
  -f Dockerfile.linux `
  --build-arg BIN_NAME=$bin `
  --output "type=local,dest=$dist" `
  .

Write-Host "Done. Linux binary: release\linux\$bin" -ForegroundColor Green
