$port = 3000
Write-Host "Starting code-rag server on port $port..."
$process = Start-Process -FilePath "./target/release/code-rag.exe" -ArgumentList "serve", "--port", "$port" -PassThru -NoNewWindow
Start-Sleep -Seconds 10 # Wait for model loading

try {
    # Health Check
    Write-Host "Checking /health..."
    try {
        $health = Invoke-RestMethod -Uri "http://localhost:$port/health" -Method Get
        Write-Host "Health Check: $($health.status)"
        if ($health.status -ne "ok") { throw "Health check failed" }
    }
    catch {
        Write-Error "Health check request failed: $_"
        throw
    }

    # Search
    Write-Host "Testing /search..."
    $body = @{
        query = "main function"
        limit = 2
    } | ConvertTo-Json

    try {
        $results = Invoke-RestMethod -Uri "http://localhost:$port/search" -Method Post -Body $body -ContentType "application/json"
        Write-Host "Search Results Found: $($results.results.Count)"
        if ($results.results.Count -eq 0) { Write-Warning "No results found. (Note: Ensure 'index' has been run first)" }
        else {
            $results.results | ForEach-Object { Write-Host "- Found in: $($_.filename)" }
        }
    }
    catch {
        Write-Error "Search request failed: $_"
        throw
    }
}
catch {
    Write-Error "Test failed."
    exit 1
}
finally {
    Write-Host "Stopping server..."
    Stop-Process -Id $process.Id -Force
}
