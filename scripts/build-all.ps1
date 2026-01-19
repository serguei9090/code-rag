$ErrorActionPreference = "Stop"

& "$PSScriptRoot\build-linux.ps1"
& "$PSScriptRoot\build-linux-cuda.ps1"
& "$PSScriptRoot\build-windows.ps1"

Write-Host "`nAll builds complete." -ForegroundColor Green
