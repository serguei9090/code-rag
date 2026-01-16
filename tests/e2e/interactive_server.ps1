# Interactive Server Search Tool
# Usage: powershell -File tests/interactive_server.ps1

$port = 3000
$bin = "./target/debug/code-rag.exe"

if (-not (Test-Path $bin)) {
    Write-Warning "Debug binary not found, trying release..."
    $bin = "./target/release/code-rag.exe"
    if (-not (Test-Path $bin)) {
        Write-Error "Binary not found. Run 'cargo build' first."
        exit 1
    }
}

Write-Host "Starting code-rag server on port $port..." -ForegroundColor Cyan
$process = Start-Process -FilePath $bin -ArgumentList "serve", "--port", "$port" -PassThru -NoNewWindow
Start-Sleep -Seconds 5

try {
    Write-Host "`nServer ready! Type 'exit' to quit." -ForegroundColor Green
    while ($true) {
        $query = Read-Host "Enter Query"
        if ($query -eq "exit") { break }
        if ([string]::IsNullOrWhiteSpace($query)) { continue }
        
        try {
            $body = @{ query = $query; limit = 3 } | ConvertTo-Json
            $results = Invoke-RestMethod -Uri "http://localhost:$port/search" -Method Post -Body $body -ContentType "application/json"
            
            if ($results.results.Count -eq 0) {
                Write-Warning "No results found."
            }
            else {
                foreach ($res in $results.results) {
                    Write-Host "`n[$($res.score.ToString("F4"))] $($res.filename):$($res.line_start)" -ForegroundColor Yellow
                    $codeSnippet = $res.code.Split("`n") | Select-Object -First 3
                    Write-Host ($codeSnippet -join "`n") -ForegroundColor Gray
                    Write-Host "---"
                }
            }
        }
        catch {
            Write-Error "Search failed: $_"
        }
    }
}
finally {
    Write-Host "Stopping server..."
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
}
