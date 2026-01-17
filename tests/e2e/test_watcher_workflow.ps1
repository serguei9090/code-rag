$ErrorActionPreference = "Stop"
Write-Host "Starting Watcher Workflow Test..."
$env:RUST_LOG = "debug"

# Clean previous
if (Test-Path ".lancedb") { Remove-Item -Recurse -Force ".lancedb" }
if (Test-Path "test_data") { Remove-Item -Recurse -Force "test_data" }

# Create test data
New-Item -ItemType Directory -Force "test_data"
"initial content" | Set-Content "test_data\file1.rs"

# Build (Already built)
# cargo build 

# Index
Write-Host "Indexing..."
& .\target\debug\code-rag.exe index --path test_data

# Start Watcher in background
Write-Host "Starting Watcher..."
$watcher = Start-Process -FilePath ".\target\debug\code-rag.exe" -ArgumentList "watch", "--path", "test_data" -PassThru -NoNewWindow
Write-Host "Waiting 15s for watcher to initialize..."
Start-Sleep -Seconds 15

# Modify file
Write-Host "Modifying file..."
"updated content" | Set-Content "test_data\file1.rs"
Write-Host "Waiting 5s for watcher to process..."
Start-Sleep -Seconds 5

# Search
Write-Host "Searching for 'updated'..."
$output = & .\target\debug\code-rag.exe search "updated"
Write-Host "Search Output: $output"

# Cleanup Watcher
Stop-Process -Id $watcher.Id -Force

if ($output -match "file1.txt") {
    Write-Host "Test PASSED: 'updated' found in file1.txt"
    exit 0
} else {
    Write-Host "Test FAILED: 'updated' not found in search results."
    exit 1
}
