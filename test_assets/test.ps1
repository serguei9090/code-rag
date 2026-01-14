# PowerShell test script

$GlobalConfig = @{
    Path = "C:\Logs"
}

# function_definition
function Get-SystemStatus {
    param($Detailed)
    
    if ($Detailed) {
        Write-Host "Fetching detailed status..."
    }
    return "OK"
}

# if_expression (depth 1)
if ($null -eq $GlobalConfig) {
    Write-Error "Config missing"
}

# call / command (depth 1)
Get-SystemStatus -Detailed
