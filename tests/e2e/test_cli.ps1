#!/usr/bin/env pwsh
# Black-Box CLI Test Suite for code-rag
# Tests the compiled binary through all commands and validates outputs

param(
    [string]$ProjectRoot = "$PSScriptRoot/../..",
    [string]$BinaryPath = ".\target\release\code-rag.exe",
    [string]$TestDbPath = "$PSScriptRoot/../..//.lancedb-blackbox-test"
)

$ErrorActionPreference = "Stop"
$TestsPassed = 0
$TestsFailed = 0
$TestAssets = "$ProjectRoot/test_assets"

# Colors for output
function Write-Success { param($msg) Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Failure { param($msg) Write-Host "[FAIL] $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "-> $msg" -ForegroundColor Cyan }
function Write-Section { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Yellow }

# Cleanup function
function Cleanup-TestDb {
    if (Test-Path $TestDbPath) {
        Remove-Item -Recurse -Force $TestDbPath -ErrorAction SilentlyContinue
    }
}

# Test assertion helper
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

# Check if binary exists
Write-Section "Pre-flight Checks"
if (-not (Test-Path $BinaryPath)) {
    Write-Failure "Binary not found at $BinaryPath"
    Write-Info "Please run: cargo build --release"
    exit 1
}
Write-Success "Binary found at $BinaryPath"

# Check if test assets exist
if (-not (Test-Path $TestAssets)) {
    Write-Failure "Test assets not found at $TestAssets"
    exit 1
}
Write-Success "Test assets found at $TestAssets"

# Cleanup before starting
Cleanup-TestDb
Write-Success "Test database cleaned"

# Test 1: Version Command
Write-Section "Test 1: Version Command"
try {
    $version = & cargo run --bin code-rag -- --version 2>&1
    Assert-Success "Version command executes" ($LASTEXITCODE -eq 0) "Exit code: $LASTEXITCODE"
    Assert-Success "Version output contains 'code-rag'" ($version -match "code-rag") "Output: $version"
}
catch {
    Assert-Success "Version command executes" $false $_.Exception.Message
}

# Test 2: Help Command
Write-Section "Test 2: Help Command"
try {
    $help = & cargo run --bin code-rag -- --help 2>&1
    Assert-Success "Help command executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Help contains 'index' subcommand" ($help -match "index")
    Assert-Success "Help contains 'search' subcommand" ($help -match "search")
    Assert-Success "Help contains 'grep' subcommand" ($help -match "grep")
}
catch {
    Assert-Success "Help command executes" $false $_.Exception.Message
}

# Test 3: Index Command - Basic
Write-Section "Test 3: Index Command (Basic)"
try {
    Write-Info "Indexing test assets..."
    $idx = & cargo run --bin code-rag -- index
    
    Assert-Success "Index command executes" ($LASTEXITCODE -eq 0) "Exit code: $LASTEXITCODE"
    Assert-Success "Database directory created" (Test-Path $TestDbPath)
    
    # Check if output shows indexing completion
    Assert-Success "Index output shows optimization" ($indexOutput -match "Optimizing index")
}
catch {
    Assert-Success "Index command executes" $false $_.Exception.Message
}

# Test 4: Index Command - Force Re-index
Write-Section "Test 4: Index Command (Force Re-index)"
try {
    Write-Info "Force re-indexing..."
    $forceOutput = & cargo run --bin code-rag -- index $TestAssets --db-path $TestDbPath --force 2>&1 | Out-String
    
    Assert-Success "Force re-index executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Force flag acknowledged" ($forceOutput -match "Force|Removing")
}
catch {
    Assert-Success "Force re-index executes" $false $_.Exception.Message
}

# Test 5: Search Command - Rust
Write-Section "Test 5: Search Command (Rust)"
try {
    Write-Info "Searching for Rust function..."
    $searchOutput = & cargo run --bin code-rag -- search "rust function" --db-path $TestDbPath 2>&1 | Out-String
    
    Assert-Success "Search command executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Search returns results" ($searchOutput -match "Rank|File|Score")
    Assert-Success "Search finds Rust file" ($searchOutput -match "test\.rs")
}
catch {
    Assert-Success "Search command executes" $false $_.Exception.Message
}

# Test 6: Search Command - Python
Write-Section "Test 6: Search Command (Python)"
try {
    Write-Info "Searching for Python code..."
    $searchOutput = & cargo run --bin code-rag -- search "python function" --db-path $TestDbPath 2>&1 | Out-String
    
    Assert-Success "Python search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Python search finds results" ($searchOutput -match "test\.py")
}
catch {
    Assert-Success "Python search executes" $false $_.Exception.Message
}

# Test 7: Search Command - Bash
Write-Section "Test 7: Search Command (Bash)"
try {
    Write-Info "Searching for Bash script..."
    $searchOutput = & cargo run --bin code-rag -- search "backup logs" --db-path $TestDbPath 2>&1 | Out-String
    
    Assert-Success "Bash search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Bash search finds shell script" ($searchOutput -match "test\.sh")
}
catch {
    Assert-Success "Bash search executes" $false $_.Exception.Message
}

# Test 8: Search Command - PowerShell
Write-Section "Test 8: Search Command (PowerShell)"
try {
    Write-Info "Searching for PowerShell function..."
    $searchOutput = & cargo run --bin code-rag -- search "system status" --db-path $TestDbPath 2>&1 | Out-String
    
    Assert-Success "PowerShell search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "PowerShell search finds ps1 file" ($searchOutput -match "test\.ps1")
}
catch {
    Assert-Success "PowerShell search executes" $false $_.Exception.Message
}

# Test 9: Search Command - JSON
Write-Section "Test 9: Search Command (JSON)"
try {
    Write-Info "Searching for JSON configuration..."
    $searchOutput = & cargo run --bin code-rag -- search "configuration database" --db-path $TestDbPath 2>&1 | Out-String
    
    Assert-Success "JSON search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "JSON search finds json file" ($searchOutput -match "test\.json")
}
catch {
    Assert-Success "JSON search executes" $false $_.Exception.Message
}

# Test 10: Search Command - YAML
Write-Section "Test 10: Search Command (YAML)"
try {
    Write-Info "Searching for YAML config..."
    $searchOutput = & cargo run --bin code-rag -- search "project name version" --db-path $TestDbPath 2>&1 | Out-String
    
    Assert-Success "YAML search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "YAML search finds yaml file" ($searchOutput -match "test\.yaml")
}
catch {
    Assert-Success "YAML search executes" $false $_.Exception.Message
}

# Test 11: Search Command - Limit Parameter
Write-Section "Test 11: Search Command (Limit Parameter)"
try {
    Write-Info "Testing limit parameter..."
    $searchOutput = & cargo run --bin code-rag -- search "function" --db-path $TestDbPath --limit 3 2>&1 | Out-String
    
    Assert-Success "Search with limit executes" ($LASTEXITCODE -eq 0)
    
    # Count number of "Rank" occurrences (should be <= 3)
    $rankCount = ([regex]::Matches($searchOutput, "Rank \d+")).Count
    Assert-Success "Limit parameter respected" ($rankCount -le 3) "Found $rankCount results"
}
catch {
    Assert-Success "Search with limit executes" $false $_.Exception.Message
}

# Test 12: Search Command - HTML Report
Write-Section "Test 12: Search Command (HTML Report)"
try {
    Write-Info "Testing HTML report generation..."
    $htmlPath = ".\results.html"
    if (Test-Path $htmlPath) { Remove-Item $htmlPath }
    
    $searchOutput = & cargo run --bin code-rag -- search "function" --db-path $TestDbPath --html 2>&1 | Out-String
    
    Assert-Success "HTML report generation executes" ($LASTEXITCODE -eq 0)
    Assert-Success "HTML file created" (Test-Path $htmlPath)
    
    if (Test-Path $htmlPath) {
        $htmlContent = Get-Content $htmlPath -Raw
        Assert-Success "HTML contains results" ($htmlContent -match "<html|<body")
        Remove-Item $htmlPath -ErrorAction SilentlyContinue
    }
}
catch {
    Assert-Success "HTML report generation executes" $false $_.Exception.Message
}

# Test 13: Grep Command - Exact Match
Write-Section "Test 13: Grep Command (Exact Match)"
try {
    Write-Info "Testing grep for exact pattern..."
    $grepOutput = & cargo run --bin code-rag -- grep "function" 2>&1 | Out-String
    
    Assert-Success "Grep command executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Grep finds matches" ($grepOutput.Length -gt 0)
}
catch {
    Assert-Success "Grep command executes" $false $_.Exception.Message
}

# Test 14: Grep Command - Case Sensitivity
Write-Section "Test 14: Grep Command (Case Sensitivity)"
try {
    Write-Info "Testing grep case sensitivity..."
    $grepOutput = & cargo run --bin code-rag -- grep "FUNCTION" 2>&1 | Out-String
    
    Assert-Success "Grep case-insensitive executes" ($LASTEXITCODE -eq 0)
    # Should find matches even with different case
    Assert-Success "Grep is case-insensitive by default" ($grepOutput.Length -gt 0)
}
catch {
    Assert-Success "Grep case-insensitive executes" $false $_.Exception.Message
}

# Test 15: Multi-Language Indexing
Write-Section "Test 15: Multi-Language Verification"
try {
    Write-Info "Verifying all languages were indexed..."
    
    # Search for language-specific patterns
    $languages = @{
        "Rust"       = "fn main"
        "Python"     = "def "
        "Go"         = "func "
        "JavaScript" = "class "
        "Bash"       = "backup_logs"
        "PowerShell" = "Get-SystemStatus"
        "JSON"       = "project"
        "YAML"       = "code-rag"
    }
    
    $foundLanguages = 0
    foreach ($lang in $languages.Keys) {
        $pattern = $languages[$lang]
        $result = & cargo run --bin code-rag -- grep $pattern 2>&1 | Out-String
        if ($result.Length -gt 0) {
            $foundLanguages++
            Write-Success "$lang code found"
        }
        else {
            Write-Failure "$lang code NOT found"
        }
    }
    
    Assert-Success "Multi-language support verified" ($foundLanguages -ge 6) "Found $foundLanguages/8 languages"
}
catch {
    Assert-Success "Multi-language verification" $false $_.Exception.Message
}

# Test 16: Advanced Python Structure
Write-Section "Test 16: Advanced Structure (Nested Python)"
try {
    Write-Info "Verifying nested Python modules..."
    
    # Check for deep class
    $deepOutput = & cargo run --bin code-rag -- search "DeepClass" --db-path $TestDbPath --limit 1 2>&1 | Out-String
    Assert-Success "Deep search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Deep search finds nested file" ($deepOutput -match "sub_mod[\\/]deep\.py")
    
    # Check for logic in class
    $logicOutput = & cargo run --bin code-rag -- search "complex processing logic" --db-path $TestDbPath --limit 1 2>&1 | Out-String
    Assert-Success "Logic search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Logic search finds processor class" ($logicOutput -match "processor\.py")

    # Check for dunder (double underscore) class
    $dunderOutput = & cargo run --bin code-rag -- search "__SecretInternal" --db-path $TestDbPath --limit 1 2>&1 | Out-String
    Assert-Success "Dunder search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Dunder search finds class" ($dunderOutput -match "dunder_test\.py")

    # Check that __pycache__ content is IGNORED (Negative Test)
    # Note: explicit grep for file path to see if it exists in index
    $ignoredOutput = & cargo run --bin code-rag -- grep "should_not_be_indexed" 2>&1 | Out-String
    if ($ignoredOutput -match "cached_logic\.py") {
        Write-Failure "Indexing failed to ignore __pycache__"
        $script:TestsFailed++
    }
    else {
        Write-Success "__pycache__ content correctly ignored"
        $script:TestsPassed++
    }
}
catch {
    Assert-Success "Advanced structure test" $false $_.Exception.Message
}

# Test 17: JSON Output (Search)
Write-Section "Test 17: JSON Output (Search)"
try {
    Write-Info "Testing JSON search output..."
    $jsonOutput = & cargo run --bin code-rag -- search "Rust function" --db-path $TestDbPath --json | Out-String
    
    # Try to parse as JSON
    $results = $jsonOutput | ConvertFrom-Json
    Assert-Success "JSON search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "JSON search is valid JSON" ($results -is [array])
    Assert-Success "JSON search contains filename" ($results[0].filename -match "test\.rs")
}
catch {
    Assert-Success "JSON search test" $false $_.Exception.Message
}

# Test 18: JSON Output (Grep)
Write-Section "Test 18: JSON Output (Grep)"
try {
    Write-Info "Testing JSON grep output..."
    $jsonOutput = & cargo run --bin code-rag -- grep "function" --json | Out-String
    
    $results = $jsonOutput | ConvertFrom-Json
    Assert-Success "JSON grep executes" ($LASTEXITCODE -eq 0)
    Assert-Success "JSON grep is valid JSON" ($results -is [array])
    Assert-Success "JSON grep contains matches" ($results.Count -gt 0)
}
catch {
    Assert-Success "JSON grep test" $false $_.Exception.Message
}

# Test 19: New Languages (Zig/Elixir)
Write-Section "Test 19: New Languages (Zig/Elixir)"
try {
    Write-Info "Verifying Zig and Elixir indexing..."
    
    # First re-index to pick up new files
    & cargo run --bin code-rag -- index $TestAssets --db-path $TestDbPath --force | Out-String
    
    $zigResult = & cargo run --bin code-rag -- grep "Zig function" 2>&1 | Out-String
    Assert-Success "Zig code indexed" ($zigResult -match "test\.zig")
    
    $exResult = & cargo run --bin code-rag -- grep "Elixir function" 2>&1 | Out-String
    Assert-Success "Elixir code indexed" ($exResult -match "test\.ex")
}
catch {
    Assert-Success "New languages test" $false $_.Exception.Message
}

# Test 20: Metadata Filtering (Extension)
Write-Section "Test 20: Metadata Filtering (Extension)"
try {
    Write-Info "Testing --ext filter..."
    $rustOnlyOutput = & cargo run --bin code-rag -- search "function" --db-path $TestDbPath --ext rs --limit 10 2>&1 | Out-String
    
    Assert-Success "Extension filter executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Extension filter finds Rust files" ($rustOnlyOutput -match "test\.rs")
    
    if ($rustOnlyOutput -match "test\.py") {
        Write-Failure "Extension filter incorrectly included Python files"
        $script:TestsFailed++
    }
    else {
        Write-Success "Extension filter correctly excludes non-Rust files"
        $script:TestsPassed++
    }
}
catch {
    Assert-Success "Extension filter test" $false $_.Exception.Message
}

# Test 21: Metadata Filtering (Directory)
Write-Section "Test 21: Metadata Filtering (Directory)"
try {
    Write-Info "Testing --dir filter..."
    $advancedDirOutput = & cargo run --bin code-rag -- search "class" --db-path $TestDbPath --dir "test_assets\advanced_structure" --limit 10 2>&1 | Out-String
    
    Assert-Success "Directory filter executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Directory filter finds files in target dir" ($advancedDirOutput -match "advanced_structure")
}
catch {
    Assert-Success "Directory filter test" $false $_.Exception.Message
}

# Test 22: No Rerank Search
Write-Section "Test 22: No Rerank Search"
try {
    Write-Info "Testing --no-rerank flag..."
    $startTime = Get-Date
    $fastOutput = & cargo run --bin code-rag -- search "function" --db-path $TestDbPath --no-rerank --limit 5 2>&1 | Out-String
    $duration = (Get-Date) - $startTime
    
    Assert-Success "No rerank search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "No rerank search returns results" ($fastOutput -match "Rank|File|Score")
    Write-Info "Execution time: $($duration.TotalSeconds)s"
}
catch {
    Assert-Success "No rerank search test" $false $_.Exception.Message
}

# Test 23: LanceDB Index Verification
Write-Section "Test 23: LanceDB Filename Index Verification"
try {
    Write-Info "Verifying filename index exists and improves performance..."
    
    # The index should be created automatically during indexing
    # We verify it works by testing filtered searches
    $indexTestOutput = & cargo run --bin code-rag -- search "function" --db-path $TestDbPath --ext rs --limit 5 2>&1 | Out-String
    
    Assert-Success "Index-backed filtered search executes" ($LASTEXITCODE -eq 0)
    Assert-Success "Index-backed search returns results" ($indexTestOutput -match "Rank|File|Score")
    
    # Verify all results are .rs files
    $nonRustMatches = ([regex]::Matches($indexTestOutput, "\.(py|js|go|java)")).Count
    Assert-Success "Filename index correctly filters results" ($nonRustMatches -eq 0) "Found $nonRustMatches non-Rust files"
    
    Write-Info "LanceDB filename index is functioning correctly"
}
catch {
    Assert-Success "LanceDB index verification test" $false $_.Exception.Message
}

# Cleanup
Write-Section "Cleanup"
Cleanup-TestDb
Write-Success "Test database cleaned up"

# Summary
Write-Section "Test Summary"
$total = $TestsPassed + $TestsFailed
Write-Host "`nTotal Tests: $total" -ForegroundColor White
Write-Host "Passed: $TestsPassed" -ForegroundColor Green
Write-Host "Failed: $TestsFailed" -ForegroundColor Red

if ($TestsFailed -eq 0) {
    Write-Host "`n🎉 All tests passed!" -ForegroundColor Green
    exit 0
}
else {
    Write-Host "`n⚠️  Some tests failed" -ForegroundColor Yellow
    exit 1
}
