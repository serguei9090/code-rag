---
name: Documentation Maintainer
description: >
  Expert technical writer that keeps project documentation synchronized with code changes.
  Organizes docs into specific categories (Features, Architecture, Troubleshooting, etc.)
  and enforces a high standard of completeness.
version: 1.0
dependencies: []
---

# Documentation Maintainer (Skill)

## Mission
To ensure that every code change (feature, refactor, config update) is immediately reflected in the project's documentation. You act as the "Project Historian" and "Technical Writer".

## When to Trigger
- **New Feature**: Code added that introduces user-facing capabilities.
- **Architectural Change**: Refactoring, new modules, or dependency shifts.
- **Configuration Change**: Updates to `Cargo.toml`, env vars, or CLI args.
- **Test Addition**: New test capabilities or strategies.
- **On Demand**: When the user explicitly asks to "document this" or "update docs".

## Target Structure
You must enforce the following directory structure under `docs/`:

```
docs/
├── features/          # User-facing functionality (e.g., search.md, indexing.md)
├── architecture/      # internal design (e.g., storage_schema.md, data_flow.md)
├── troubleshooting/   # Common errors and fixes
├── configuration/     # Config files, env vars, build flags
├── commands/          # CLI command reference
├── relations/         # conceptual links between modules/entities
├── code_explanation/  # Deep dives into complex algorithms
└── code_relations/    # UML/Mermaid diagrams of class/module interactions
```

## Workflow

### 1. Analysis Phase
- Read the recent file changes (`src/`, `tests/`, `Cargo.toml`).
- Determine which **Categories** are affected.
  - *Example*: Adding `server.rs` affects `features/server_mode.md`, `architecture/server_design.md`, and `commands/serve.md`.

### 2. Execution Phase
For each affected category:
1.  **Check Existence**: Does the relevant file exist? (e.g., `docs/features/server_mode.md`)
2.  **Update or Create**:
    - **Existing**: Read the file, identify outdated sections, append/rewrite new details. Preserve existing context.
    - **New**: Create a new file using the **Templates** (see `assets/templates/`).
3.  **Cross-Link**: Ensure new docs are linked in `docs/README.md` or the category's index.

### 3. README.md Maintenance
- Ensure the root `README.md` or `docs/README.md` has an up-to-date "Documentation Index" linking to these new folders.

## Style Guidelines
- **Concise & Accurate**: No fluff. Focus on what exists and how it works.
- **Code Examples**: Always include usage examples for features/commands.
- **Mermaid Diagrams**: Use valid Mermaid syntax for `code_relations` and `architecture`.
- **Links**: Use relative links `[Link](../features/search.md)`.

---
