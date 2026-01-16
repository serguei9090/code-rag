#!/usr/bin/env pwsh
# Master Test Runner for code-rag

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot

$TestSuites = @(
    @{ Name = "CLI Functional Tests"; File = "$ScriptRoot/test_cli.ps1" },
    @{ Name = "Server Basic Tests";   File = "$ScriptRoot/test_server.ps1" },
    @{ Name = "Concurrency & Stress"; File = "$ScriptRoot/test_concurrent_ops.ps1" }
)

$TotalTestsPassed = 0
$TotalTestsFailed = 0
$Results = @()

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "  Code-Rag Automated Test Suite" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Date: $(Get-Date)"

$TotalTimer = [System.Diagnostics.Stopwatch]::StartNew()

foreach ($suite in $TestSuites) {
    $name = $suite.Name
    $file = $suite.File
    Write-Host "Running: $name..." -ForegroundColor Yellow
    
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $exitCode = 0
    try {
        if (-not (Test-Path $file)) {
            throw "File not found: $file"
        }
        # Execute the test script
        & $file | Out-Default
        $exitCode = $LASTEXITCODE
    }
    catch {
        Write-Error "Failed to execute $file : $_"
        $exitCode = 1
    }
    $sw.Stop()
    
    $status = "FAIL"
    if ($exitCode -eq 0) { 
        $status = "PASS" 
        $TotalTestsPassed++
    } else {
        $TotalTestsFailed++
    }
    
    $durationStr = $sw.Elapsed.ToString()
    
    $Results += [PSCustomObject]@{
        Suite    = $name
        Status   = $status
        Duration = $durationStr
    }
    
    $seconds = $sw.Elapsed.TotalSeconds
    Write-Host "Finished $name in $seconds seconds [$status]`n" -ForegroundColor Gray
}

$TotalTimer.Stop()

# Final Report
Write-Host "`n==================================================" -ForegroundColor Cyan
Write-Host "  Test Execution Summary" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

$Results | Format-Table -AutoSize

Write-Host "Total Duration: $($TotalTimer.Elapsed.ToString())"
Write-Host "Suites Passed:  $TotalTestsPassed / $($TestSuites.Count)"

if ($TotalTestsFailed -eq 0) {
    Write-Host "`nAll Test Suites Passed" -ForegroundColor Green
    exit 0
}
else {
    Write-Host "`nSome Test Suites Failed" -ForegroundColor Red
    exit 1
}
