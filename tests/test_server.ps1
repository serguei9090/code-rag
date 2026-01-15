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

    # Test Error Handling - Invalid JSON
    Write-Host "Testing error handling (invalid JSON)..."
    try {
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:$port/search" -Method Post -Body "invalid json" -ContentType "application/json" -ErrorAction Stop
            Write-Error "Expected error response, but request succeeded"
            throw
        }
        catch [Microsoft.PowerShell.Commands.HttpResponseException] {
            $statusCode = $_.Exception.Response.StatusCode.value__
            Write-Host "Received expected error status: $statusCode"
            if ($statusCode -ne 400 -and $statusCode -ne 500) {
                Write-Error "Expected status 400 or 500, got $statusCode"
                throw
            }
            Write-Host "✓ Error handling test passed (invalid JSON)"
        }
    }
    catch {
        Write-Error "Error handling test failed: $_"
        throw
    }

    # Test Error Handling - Missing Query Field
    Write-Host "Testing error handling (missing required field)..."
    try {
        $badBody = @{ limit = 5 } | ConvertTo-Json
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:$port/search" -Method Post -Body $badBody -ContentType "application/json" -ErrorAction Stop
            Write-Error "Expected error response, but request succeeded"
            throw
        }
        catch [Microsoft.PowerShell.Commands.HttpResponseException] {
            $statusCode = $_.Exception.Response.StatusCode.value__
            Write-Host "Received expected error status: $statusCode"
            if ($statusCode -ne 400 -and $statusCode -ne 500) {
                Write-Error "Expected status 400 or 500, got $statusCode"
                throw
            }
            Write-Host "✓ Error handling test passed (missing required field)"
        }
    }
    catch {
        Write-Error "Error handling test failed (missing field): $_"
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
