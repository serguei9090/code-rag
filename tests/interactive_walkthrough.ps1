$BinaryPath = ".\target\release\code-rag.exe"
$DbPath = ".\.lancedb-interactive"
$Global:ErrorActionPreference = "Stop"

function Pause-Step {
    param([string]$Message)
    Write-Host "`n--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "step: $Message" -ForegroundColor Yellow
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Read-Host "Press Enter to execute this step (or Ctrl+C to abort)..."
}

function Invoke-Timed {
    param([scriptblock]$Command)
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    & $Command | Out-Default # Ensure output goes to console, not pipeline
    $timer.Stop()
    return $timer.Elapsed.TotalSeconds
}

function Show-Result {
    param([double]$Seconds)
    Write-Host "`n✅ Step Output (Time: $($Seconds.ToString("F3"))s)" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Gray
    Read-Host "Inspect output above. Press Enter to continue to next step..." | Out-Null
    Write-Host "`n"
}

Write-Host "interactive walkthrough for code-rag" -ForegroundColor Magenta
Write-Host "binary: $BinaryPath"
Write-Host "db path: $DbPath"

# 0. Clean old DB
if (Test-Path $DbPath) {
    Remove-Item -Recurse -Force $DbPath
    Write-Host "Cleaned up old test database." -ForegroundColor Gray
}

# 1. Help
Pause-Step "Show Help screen"
$t = Invoke-Timed { & $BinaryPath --help }
Show-Result $t

# 2. Index
Pause-Step "Index test assets (./test_assets)"
$t = Invoke-Timed { & $BinaryPath index .\test_assets --db-path $DbPath }
Show-Result $t

# 3. Basic Search
Pause-Step "Search for 'function' (Natural Language)"
$t = Invoke-Timed { & $BinaryPath search "function" --db-path $DbPath --limit 3 }
Show-Result $t

# 4. Filtered Search (Extension)
Pause-Step "Search for 'function' restricted to Rust files (--ext rs)"
$t = Invoke-Timed { & $BinaryPath search "function" --db-path $DbPath --ext rs --limit 3 }
Show-Result $t

# 5. Filtered Search (Directory)
Pause-Step "Search for 'class' restricted to 'advanced_structure' folder (--dir)"
$t = Invoke-Timed { & $BinaryPath search "class" --db-path $DbPath --dir "test_assets\advanced_structure" --limit 3 }
Show-Result $t

# 6. JSON Output
Pause-Step "Search with JSON output (--json)"
$t = Invoke-Timed { & $BinaryPath search "function" --db-path $DbPath --limit 1 --json }
Show-Result $t

# 7. Fast Search (No Rerank)
Pause-Step "Fast Search without Re-ranking (--no-rerank)"
$t = Invoke-Timed { & $BinaryPath search "function" --db-path $DbPath --limit 5 --no-rerank }
Show-Result $t

# 8. Grep (Exact Match)
Pause-Step "Grep search for exact string 'tokio::main'"
$t = Invoke-Timed { & $BinaryPath grep "tokio::main" }
Show-Result $t

# End
Write-Host "Walkthrough complete! deleting temp db..." -ForegroundColor Magenta
if (Test-Path $DbPath) {
    Remove-Item -Recurse -Force $DbPath
}
Write-Host "Done." -ForegroundColor Green
