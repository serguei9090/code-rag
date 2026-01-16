$ErrorActionPreference = "Stop"

Write-Host "Starting Server..."
$p = Start-Process -FilePath "target/debug/code-rag.exe" -ArgumentList "serve", "--port", "3333" -PassThru -NoNewWindow
Start-Sleep -Seconds 10

try {
    Write-Host "Checking Health..."
    $resp = Invoke-RestMethod "http://localhost:3333/health"
    if ($resp.status -eq "ok") {
        Write-Host "SUCCESS: Health check passed."
    } else {
        Write-Host "FAILURE: Health check failed."
        exit 1
    }
} catch {
    Write-Host "FAILURE: Could not connect to server: $_"
    exit 1
} finally {
    Stop-Process -Id $p.Id
}
