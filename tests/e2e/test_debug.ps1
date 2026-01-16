#!/usr/bin/env pwsh
# Debug Test Suite for code-rag
# Troubleshooting specific failures in Tests 9, 10, 16, 17, 20, 23

param(
    [string]$ProjectRoot = "$PSScriptRoot/../..",
    [string]$TestDbPath = "$PSScriptRoot/../..//.lancedb-blackbox-debug"
)

$ErrorActionPreference = "Continue"
$TestAssets = "$ProjectRoot/test_assets"

# Colors
function Write-Header { param($msg) Write-Host "`n=== DEBUG: $msg ===" -ForegroundColor Yellow }
function Write-Info { param($msg) Write-Host "INFO: $msg" -ForegroundColor Cyan }
function Write-ErrorMsg { param($msg) Write-Host "ERROR: $msg" -ForegroundColor Red }

# Ensure fresh start
Write-Header "Environment Setup"
if (Test-Path $TestDbPath) {
    Write-Info "Cleaning up old debug DB at $TestDbPath"
    Remove-Item -Recurse -Force $TestDbPath -ErrorAction SilentlyContinue
}

# 1. Indexing
Write-Header "Step 1: Indexing Test Assets"
Write-Info "Assets Path: $TestAssets"
Write-Info "DB Path: $TestDbPath"

$indexCmd = "cargo"
$indexArgs = @("run", "--quiet", "--bin", "code-rag", "--", "index", "$TestAssets", "--db-path", "$TestDbPath")

Write-Host "Running: $indexCmd $indexArgs" -ForegroundColor Gray
& $indexCmd $indexArgs

if ($LASTEXITCODE -ne 0) {
    Write-ErrorMsg "Indexing FAILED with exit code $LASTEXITCODE. Aborting."
    exit $LASTEXITCODE
} else {
    Write-Info "Indexing completed successfully."
}

# Check if DB directory exists
if (-not (Test-Path $TestDbPath)) {
    Write-ErrorMsg "DB Directory was NOT created at $TestDbPath"
    exit 1
}
Write-Info "DB Directory exists."

# 2. Debug JSON/YAML Search
Write-Header "Step 2: JSON & YAML Search Analysis"

# Check if assets exist
$jsonFiles = Get-ChildItem -Recurse $TestAssets -Filter "*.json"
$yamlFiles = Get-ChildItem -Recurse $TestAssets -Filter "*.yaml"

Write-Info "Found $($jsonFiles.Count) JSON files in assets"
$jsonFiles | ForEach-Object { Write-Host " - $($_.FullName)" -ForegroundColor Gray }
Write-Info "Found $($yamlFiles.Count) YAML files in assets"
$yamlFiles | ForEach-Object { Write-Host " - $($_.FullName)" -ForegroundColor Gray }

Write-Info "Running JSON Search command..."
& cargo run --quiet --bin code-rag -- search "configuration database" --db-path $TestDbPath | Out-Host

Write-Info "Running YAML Search command..."
& cargo run --quiet --bin code-rag -- search "project name version" --db-path $TestDbPath | Out-Host


# 3. Debug Advanced (Nested) Structure
Write-Header "Step 3: Nested Python Analysis"
$deepFile = "$TestAssets/advanced_structure/sub_mod/deep.py"
if (Test-Path $deepFile) {
    Write-Info "deep.py exists at $deepFile"
} else {
    Write-ErrorMsg "deep.py MISSING at $deepFile"
}

Write-Info "Searching for 'DeepClass'..."
& cargo run --quiet --bin code-rag -- search "DeepClass" --db-path $TestDbPath --limit 5 | Out-Host


# 4. Debug JSON Output & Table Error
Write-Header "Step 4: JSON Output Debug"
Write-Info "Testing JSON output command..."
$jsonOutput = & cargo run --quiet --bin code-rag -- search "rust" --db-path $TestDbPath --json
# Capture output to variable but also print to host for user
$strOutput = $jsonOutput | Out-String
Write-Host $strOutput

if ($strOutput -match "Table 'code_chunks' was not found") {
    Write-ErrorMsg "CRITICAL: Table 'code_chunks' not found error detected!"
    Write-Info "Listing DB directory contents:"
    Get-ChildItem -Recurse $TestDbPath | Select-Object FullName
}

try {
    $parsed = $strOutput | ConvertFrom-Json
    if ($parsed) {
        Write-Info "Successfully parsed JSON response."
        Write-Info "Result count: $($parsed.Count)"
    }
} catch {
    Write-ErrorMsg "Failed to parse JSON output: $_"
}


# 5. Debug Metadata Filtering (Extension)
Write-Header "Step 5: Metadata Extension Filter Debug"
Write-Info "Running search with --ext rs (Expect ONLY .rs files)..."
$extResults = & cargo run --quiet --bin code-rag -- search "function" --db-path $TestDbPath --ext rs --limit 20
$extResults | Out-Host

# Check for pollution
if ($extResults -match "\.py") {
    Write-ErrorMsg "FAIL: Found .py files in --ext rs results!"
    $extResults | Select-String "\.py" | ForEach-Object { Write-Host " - POLLUTION: $_" -ForegroundColor Red }
} else {
    Write-Info "Filtering looks correct (no .py files found)."
}
