# Contributing to Code RAG

Thank you for your interest in contributing to Code RAG!

## Development Workflow

We use [Lefthook](https://github.com/evilmartians/lefthook) to enforce code quality standards before every commit. This ensures that all code is properly formatted and linted.

### Prerequisites

1.  **Install Lefthook**
    You can install it via Cargo, NPM, or your system package manager.

    **Cargo:**
    ```bash
    cargo install lefthook
    ```

    **NPM:**
    ```bash
    npm install -g @evilmartians/lefthook
    ```

2.  **Enable Hooks**
    Once installed, run the following command in the project root to install the git hooks:

    ```bash
    lefthook install
    ```

### Pre-commit Checks

The following checks are run automatically before every commit:

-   **Formatting**: `cargo fmt --all -- --check`
-   **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`

If any of these fail, the commit will be aborted. You can auto-fix many issues by running:

```bash
cargo clippy --fix
cargo fmt
```
