#!/usr/bin/env pwsh
# Advanced Concurrency & Contention Tests for code-rag

param(
    [string]$ProjectRoot = "$PSScriptRoot/../..",
    [string]$BinaryPath = ".\target\release\code-rag.exe",
    [string]$TestDbPath = "$PSScriptRoot/../..//.lancedb-stress-test"
)

$ErrorActionPreference = "Stop"
$TestsPassed = 0
$TestsFailed = 0
$TestAssets = "$ProjectRoot/test_assets"

# Colors
function Write-Success { param($msg) Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Failure { param($msg) Write-Host "[FAIL] $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "-> $msg" -ForegroundColor Cyan }
function Write-Section { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Yellow }

function Assert-Success {
    param($TestName, $Condition, $ErrorMsg = "Test failed")
    if ($Condition) {
        Write-Success $TestName
        $script:TestsPassed++
    }
    else {
        Write-Failure "$TestName - $ErrorMsg"
        $script:TestsFailed++
    }
}

# Cleanup
if (Test-Path $TestDbPath) { Remove-Item -Recurse -Force $TestDbPath -ErrorAction SilentlyContinue }

# Ensure Binary Exists
if (-not (Test-Path $BinaryPath)) {
    Write-Failure "Binary not found at $BinaryPath"
    exit 1
}

# -------------------------------------------------------------------------
# Test 1: Server Stress Test (20+ Parallel Requests)
# -------------------------------------------------------------------------
Write-Section "Test 1: Server Stress Test"
$port = 4000
$serverProcess = Start-Process -FilePath $BinaryPath -ArgumentList "serve", "--port", "$port" -PassThru -NoNewWindow
Start-Sleep -Seconds 5 # Wait for startup

try {
    Write-Info "Sending 25 parallel search requests..."
    
    $jobs = @()
    for ($i = 0; $i -lt 25; $i++) {
        $jobs += Start-Job -ScriptBlock {
            param($currPort)
            try {
                $body = @{ query = "function"; limit = 1 } | ConvertTo-Json
                $res = Invoke-RestMethod -Uri "http://localhost:$currPort/search" -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
                return "OK"
            } catch {
                return "ERR: $_"
            }
        } -ArgumentList $port
    }

    $results = $jobs | Receive-Job -Wait
    
    $successCount = ($results | Where-Object { $_ -eq "OK" }).Count
    Assert-Success "Parallel requests handled" ($successCount -eq 25) "Successful requests: $successCount/25"
}
finally {
    Stop-Process -Id $serverProcess.Id -Force -ErrorAction SilentlyContinue
    Remove-Job $jobs -Force -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------------
# Test 2: Read/Write Contention (Simultaneous Indexing & Searching)
# -------------------------------------------------------------------------
Write-Section "Test 2: Read/Write Contention"

# 1. Initial Index
Write-Info "Creating initial index..."
& $BinaryPath index $TestAssets --db-path $TestDbPath | Out-Null

try {
    Write-Info "Starting background re-indexing (WRITE)..."
    # Start a force re-index in background
    $indexJob = Start-Job -ScriptBlock {
        param($bin, $assets, $db)
        & $bin index $assets --db-path $db --force
        return "INDEX_DONE"
    } -ArgumentList $BinaryPath, $TestAssets, $TestDbPath

    Write-Info "Executing searches during indexing (READ)..."
    # While indexing is running, hammer it with searches
    $searchFailures = 0
    $searchSuccesses = 0
    
    while ($indexJob.State -eq "Running") {
        try {
            $res = & $BinaryPath search "scan" --db-path $TestDbPath --limit 1 2>&1
            if ($LASTEXITCODE -eq 0) { $searchSuccesses++ }
            else { $searchFailures++ }
        } catch {
            $searchFailures++
        }
        Start-Sleep -Milliseconds 100
    }

    $indexResult = $indexJob | Receive-Job -Wait
    Assert-Success "Background index completed" ($indexResult -match "INDEX_DONE")
    Assert-Success "Searches succeeded during index" ($searchSuccesses -gt 0) "Successes: $searchSuccesses"
    Assert-Success "Zero search failures (graceful handling)" ($searchFailures -eq 0) "Failures: $searchFailures"
}
finally {
    Remove-Job $indexJob -Force -ErrorAction SilentlyContinue
    if (Test-Path $TestDbPath) { Remove-Item -Recurse -Force $TestDbPath -ErrorAction SilentlyContinue }
}

# Summary
Write-Section "Concurrency Test Summary"
Write-Host "Passed: $TestsPassed" -ForegroundColor Green
Write-Host "Failed: $TestsFailed" -ForegroundColor Red
if ($TestsFailed -gt 0) { exit 1 } else { exit 0 }
