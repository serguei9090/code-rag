$ErrorActionPreference = "Stop"

Write-Host "=========================================="
Write-Host "   CODE-RAG UNIFIED TEST RUNNER"
Write-Host "=========================================="

# 1. Build Release
Write-Host "`n[1/4] Building Release Binary..."
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed!"; exit 1 }

# 2. Run Unit & Integration Tests
Write-Host "`n[2/4] Running Standard Tests..."
cargo test
if ($LASTEXITCODE -ne 0) { Write-Error "Standard tests failed!"; exit 1 }

# 3. Run Performance Tests (Included but separate)
Write-Host "`n[3/4] Running Performance Tests..."
cargo test --test performance -- --ignored
if ($LASTEXITCODE -ne 0) { Write-Error "Performance tests failed!"; exit 1 }

# 4. Multi-Workspace Smoke Test
Write-Host "`n[4/4] Verifying Multi-Workspace Startup..."
# Create a temporary config for testing
$TestConfig = @"
db_path = './test_db'
default_index_path = '.'
server_host = '127.0.0.1'
server_port = 3001
enable_server = true
enable_mcp = false
enable_watch = true

[workspaces]
ProjectA = "./test_assets/ProjectA"
ProjectB = "./test_assets/ProjectB"
"@

$ConfigPath = "code-rag-test.toml"
Set-Content -Path $ConfigPath -Value $TestConfig

# Start code-rag in background
Write-Host "Starting code-rag with test config..."
$Process = Start-Process -FilePath "./target/release/code-rag.exe" -ArgumentList "--config", $ConfigPath, "start" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

if ($Process.HasExited) {
    Write-Error "code-rag process exited unexpectedly!"
    exit 1
} else {
    Write-Host "code-rag process is running (PID: $($Process.Id))"
    Stop-Process -Id $Process.Id -Force
    Write-Host "Startup verification successful."
}

# Cleanup
if (Test-Path $ConfigPath) { Remove-Item $ConfigPath }
if (Test-Path "test_db") { Remove-Item "test_db" -Recurse -Force }

Write-Host "`n=========================================="
Write-Host "   ALL TESTS PASSED SUCCESSFULLY"
Write-Host "=========================================="
