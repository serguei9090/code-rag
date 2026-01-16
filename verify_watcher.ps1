$ErrorActionPreference = "Stop"

Write-Host "Building..."
cargo build
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "Starting Watcher..."
$p = Start-Process -FilePath "target/debug/code-rag.exe" -ArgumentList "watch", "--path", "." -PassThru -NoNewWindow
Start-Sleep -Seconds 5

$func_name = "fn_watcher_verify_" + (Get-Random)
$file_name = "watcher_verify.rs"

try {
    Write-Host "Creating function: $func_name"
    "fn $func_name() {}" | Out-File $file_name -Encoding utf8
    
    # Wait for debounce and index
    Start-Sleep -Seconds 5
    
    Write-Host "Searching..."
    $res = ./target/debug/code-rag.exe search "$func_name" --json --no-rerank 2>&1
    $res | Write-Host
    
    if ($res -match $file_name) {
        Write-Host "SUCCESS: Watcher indexed the file."
    } else {
        Write-Host "FAILURE: Search did not return the file."
        exit 1
    }
} finally {
    Write-Host "Stopping Watcher..."
    Stop-Process -Id $p.Id
    Remove-Item $file_name -ErrorAction SilentlyContinue
}
