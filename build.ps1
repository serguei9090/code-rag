$ErrorActionPreference = "Stop"

function Show-Menu {
    Clear-Host
    Write-Host "===========================" -ForegroundColor Cyan
    Write-Host "   Code-RAG Build Menu" -ForegroundColor Cyan
    Write-Host "===========================" -ForegroundColor Cyan
    Write-Host "1. Build Windows (Local)"
    Write-Host "2. Build Linux (Docker)"
    Write-Host "3. Build All"
    Write-Host "Q. Quit"
    Write-Host "===========================" -ForegroundColor Cyan
}

do {
    Show-Menu
    $choice = Read-Host "Select an option"

    switch ($choice) {
        "1" {
            Write-Host "Starting Windows Build..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-windows.ps1"
            Pause
        }
        "2" {
            Write-Host "Starting Linux Build..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-linux.ps1"
            Pause
        }
        "3" {
            Write-Host "Starting All Builds..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-windows.ps1"
            & "$PSScriptRoot\scripts\build-linux.ps1"
            Pause
        }
        "q" {
            Write-Host "Exiting..." -ForegroundColor Green
            break
        }
        Default {
            Write-Host "Invalid selection. Please try again." -ForegroundColor Red
            Start-Sleep -Seconds 1
        }
    }
} until ($choice -eq "q")
