# Troubleshooting: Empty Search Results

## Problem: Search Returns Empty Results Despite Successful Indexing

### Symptoms
- `code-rag index` completes successfully and reports files processed
- `code-rag search` with the same workspace returns `[]` (empty results)
- Database directory exists and contains data files

### Example
```powershell
# Indexing succeeds
PS> code-rag index --path ./myproject --workspace myworkspace
Output: "X files processed (Indexing complete.)"

# Search returns nothing
PS> code-rag search "function" --workspace myworkspace
Output: [] (no results)
```

### Root Cause
This issue was caused by a workspace filter mismatch bug (fixed in v0.1.1+). Indexed chunks were tagged with the table name ("code_chunks") instead of the workspace identifier.

### Solution

#### If Using v0.1.1 or Later (Fixed)
The bug has been resolved. Simply re-index your workspace:

```bash
# 1. Clear the old database
rm -rf .lancedb/myworkspace

# 2. Re-index with the fixed version
code-rag index --path ./myproject --workspace myworkspace

# 3. Search should now work
code-rag search "your query" --workspace myworkspace
```

#### If Using Earlier Versions
Upgrade to v0.1.1 or later, then follow the steps above.

### Verification
After re-indexing, verify the database structure:

**Expected Structure**:
```
.lancedb/
└── myworkspace/
    ├── code_chunks.lance/    # Contains indexed data
    └── bm25_index/           # BM25 keyword index
```

Inside the database, chunks should be tagged with:
- `workspace`: `"myworkspace"` ✅ (your workspace identifier)
- NOT `workspace`: `"code_chunks"` ❌ (old bug)

### Additional Checks

1. **Confirm indexing completed**:
   ```bash
   # Check that database files exist
   ls -lh .lancedb/myworkspace/code_chunks.lance/data/
   ```

2. **Test with JSON output**:
   ```bash
   code-rag search "test" --workspace myworkspace --json
   ```
   This should return a JSON array with results, not an empty array.

3. **Try default workspace**:
   If custom workspaces fail but you need immediate functionality:
   ```bash
   # Index to default workspace
   code-rag index --path ./myproject
   
   # Search without --workspace flag
   code-rag search "your query"
   ```

### Related Issues
- Database Path Issues: See [database_not_found.md](database_not_found.md)
- Server Mode: For server-specific issues, see [Server Mode Documentation](../features/server_mode.md)

### Still Having Issues?
If search still returns empty results after re-indexing:
1. Check that the indexed path contains supported file types (`.rs`, `.py`, `.js`, etc.)
2. Verify the query matches content in your codebase
3. Try searching with `--limit 10` to get more results
4. Use `--json` output to see raw scores and metadata
