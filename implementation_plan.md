# Build Script Update Plan

## Goal
Ensure `build.ps1` and `scripts/build-windows.ps1` correctly build the `code-rag` binary locally and place it in the `release/windows` folder.

## Current State
- `scripts/build-windows.ps1`: Builds locally using `cargo` and copies to `release/windows`. **Correct.**
- `build.ps1`: Tries to use Docker for both Linux and Windows. **Incorrect for your request** (you want local native build).(this will be deleted)

## Proposed Changes

### `build.ps1`
Modify this script to act as a dispatcher or simply a wrapper for local builds.
- Remove Docker Windows build.
-   [x] Create interactive `build.ps1` script ([build.ps1](file:///i:/01-Master_Code/Test-Labs/code-rag/build.ps1))

#### [MODIFY] [build.ps1](file:///i:/01-Master_Code/Test-Labs/code-rag/build.ps1)

## Verification Plan
1.  Run `.\build.ps1`.
2.  Verify `release/windows/code-rag.exe` exists.
3.  Verify it runs (`.\release\windows\code-rag.exe --version`).
