$ErrorActionPreference = "Stop"

function Show-Menu {
    Clear-Host
    Write-Host "===========================" -ForegroundColor Cyan
    Write-Host "   Code-RAG Build Menu" -ForegroundColor Cyan
    Write-Host "===========================" -ForegroundColor Cyan
    Write-Host "1. Build Windows (Debug)"
    Write-Host "2. Build Windows (Release)"
    Write-Host "3. Build Linux (Docker)"
    Write-Host "4. Build All"
    Write-Host "Q. Quit"
    Write-Host "===========================" -ForegroundColor Cyan
}

do {
    Show-Menu
    $choice = Read-Host "Select an option"

    switch ($choice) {
        "1" {
            Write-Host "Starting Windows Build (Debug)..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-windows-debug.ps1"
            Pause
        }
        "2" {
            Write-Host "Starting Windows Build (Release)..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-windows.ps1"
            Pause
        }
        "3" {
            Write-Host "Starting Linux Build..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-linux.ps1"
            Pause
        }
        "4" {
            Write-Host "Starting All Builds..." -ForegroundColor Yellow
            & "$PSScriptRoot\scripts\build-windows-debug.ps1"
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
