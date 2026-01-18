$ErrorActionPreference = "Stop"

$bin = "code-rag"
$dist = Join-Path $PSScriptRoot "..\release\windows"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "==> Building Windows MSVC (CUDA) binary..." -ForegroundColor Cyan

# Check if CUDA feature can be built (basic check)
if (!(Test-Path Env:CUDA_PATH)) {
    Write-Host "Warning: CUDA_PATH environment variable not found. Build might fail if CUDA is not installed." -ForegroundColor Yellow
}

# Fix for VS Preview / Newer Versions (Error: unsupported Microsoft Visual Studio version)
# We strictly append the flag (supported by some wrappers) or rely on direct env var if supported.
$env:NVCC_APPEND_FLAGS = "-allow-unsupported-compiler"
$env:CUDA_NVCC_FLAGS = "-allow-unsupported-compiler"

# --- AUTOMATIC VISUAL STUDIO ENVIRONMENT SETUP ---
# nvcc needs 'cl.exe' which is only available in the VS native tools environment.
# We attempt to find and load it automatically if not already present.

if (Get-Command "cl.exe" -ErrorAction SilentlyContinue) {
    Write-Host "Create C++ compiler (cl.exe) found in PATH. Proceeding..." -ForegroundColor Cyan
} else {
    Write-Host "cl.exe not found. Attempting to load Visual Studio 2022/2019 Developer Environment..." -ForegroundColor Yellow
    
    $vsWherePath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWherePath) {
        # Find latest VS installation with VC++ tools
        $vsPath = & $vsWherePath -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        
        if ($vsPath) {
            $vcvarsScript = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
            if (Test-Path $vcvarsScript) {
                Write-Host "Found credentials script: $vcvarsScript" -ForegroundColor Cyan
                
                # Sourcing a batch file from PowerShell and keeping env vars is tricky.
                # Expected hack: Run cmd, call vcvars, then print env vars, verify and output to file, then read in PS.
                # Cleaner hack: Just run the cargo build INSIDE a cmd wrapper that calls vcvars first.
                
                Write-Host "Delegating build to CMD environment with vcvars64.bat..." -ForegroundColor Cyan
                
                # Construct the command to run in CMD: call vcvars -> cargo build
                $cargoCmd = "cargo build --release --features cuda --bin $bin"
                $cmdArgs = "/c `"`"$vcvarsScript`" && $cargoCmd`""
                
                Start-Process -FilePath "cmd.exe" -ArgumentList $cmdArgs -Wait -NoNewWindow
                
                # Verification handling checks external to this block
            } else {
                Write-Error "Found VS at $vsPath but could not find vcvars64.bat."
            }
        } else {
             Write-Host "Could not locate Visual Studio installation with vswhere. You may need to open 'x64 Native Tools Command Prompt' manually." -ForegroundColor Red
        }
    } else {
        Write-Host "vswhere.exe not found. Please run this script from the 'x64 Native Tools Command Prompt'." -ForegroundColor Red
    }
}

# Only run direct cargo build if we didn't use the cmd wrapper above (i.e., if cl.exe was already found)
if (Get-Command "cl.exe" -ErrorAction SilentlyContinue) {
    cargo build --release --features cuda --bin $bin
}


$exe = "target\release\$bin.exe"
if (!(Test-Path $exe)) {
  throw "Expected output not found: $exe"
}

$outputName = "$bin-cuda.exe"
Copy-Item $exe (Join-Path $dist $outputName) -Force

Write-Host "Done. Windows CUDA binary: release\windows\$outputName" -ForegroundColor Green
