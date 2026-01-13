# Local Requirements (Windows + Docker)

## 1) For Linux build (Docker)
- Docker Desktop for Windows
  - Enable "Use Docker Compose V2" (default)
  - BuildKit is enabled by default

To build Linux:
- Run: `.\scripts\build-linux.ps1`

## 2) For Windows build (MSVC)
### Required
1) Rust toolchain (MSVC)
- Install:
  - `winget install Rustlang.Rustup`
- Then:
  - `rustup update`
  - `rustup default stable-x86_64-pc-windows-msvc`

2) Microsoft C++ Build Tools (MSVC + Windows SDK)
- Install Visual Studio Build Tools:
  - `winget install Microsoft.VisualStudio.2022.BuildTools`
- In the installer, select workload:
  - "Desktop development with C++"
  - Ensure Windows 10/11 SDK is included

To build Windows:
- Run: `.\scripts\build-windows.ps1`

## 3) Optional but useful
- Git
- CMake (often included with VS Build Tools)
- PowerShell 7 (optional)

## 4) Output locations
- Linux: `release/linux/code-rag`
- Windows: `release/windows/code-rag.exe`
