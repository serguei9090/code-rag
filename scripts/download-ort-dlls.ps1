$ErrorActionPreference = "Stop"

$version = "1.19.2"
$url = "https://github.com/microsoft/onnxruntime/releases/download/v$version/onnxruntime-win-x64-gpu-$version.zip"
$outputZip = "onnxruntime-win-x64-gpu-$version.zip"
$extractDir = "onnxruntime-tmp"
$destination = ".\release\windows"

Write-Host "Downloading ONNX Runtime GPU v$version..."
Invoke-WebRequest -Uri $url -OutFile $outputZip

Write-Host "Extracting..."
Expand-Archive -Path $outputZip -DestinationPath $extractDir -Force

Write-Host "Copying DLLs to $destination..."
if (-not (Test-Path $destination)) {
    New-Item -ItemType Directory -Path $destination | Out-Null
}

$sourceBin = "$extractDir\onnxruntime-win-x64-gpu-$version\lib"
# Note: older zips might have bin, newer often have lib or just root.
# v1.19.2 zip structure usually: folder/lib/onnxruntime.dll AND folder/lib/onnxruntime_providers_cuda.dll
# check if 'lib' exists, if not check 'bin', if not check root.
if (-not (Test-Path $sourceBin)) {
    $sourceBin = "$extractDir\onnxruntime-win-x64-gpu-$version\bin"
    if (-not (Test-Path $sourceBin)) {
         $sourceBin = "$extractDir\onnxruntime-win-x64-gpu-$version"
    }
}

Copy-Item "$sourceBin\onnxruntime*.dll" -Destination $destination -Force
Copy-Item "$sourceBin\zlibwapi.dll" -Destination $destination -ErrorAction SilentlyContinue

Write-Host "Cleaning up..."
Remove-Item $outputZip -Force
Remove-Item $extractDir -Recurse -Force

Write-Host "Done. DLLs installed to $destination"
