# grep

## Syntax
`code-rag grep <PATTERN> [OPTIONS]`

## Overview
Performs exact text pattern matching using `ripgrep` engine embedded within the tool. Respects `.gitignore` and `.ignore` files.

## Arguments
- `<PATTERN>`: Regex pattern to search for (required)

## Options
- `--json`: Output results as JSON

## Output
List of file paths and matching lines.

## Examples

**Find function calls:**
```bash
code-rag grep "tokio::main"
```

**Find imports:**
```bash
code-rag grep "use std::"
```
