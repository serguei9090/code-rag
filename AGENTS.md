# Agent Instructions for code-rag Project

This document contains critical instructions for AI agents working on the `code-rag` codebase.

## 🔴 MANDATORY: Pre-Coding Checklist

**BEFORE writing ANY code**, you MUST:

1. **Read Project Documentation**:
   - Review `.agent/rules/rust_doc.md` for Rust coding standards
   - Check relevant documentation in `docs/` folder
   - Reference `MEMORY[rust_bp.md]` for Rust best practices
   - Review `MEMORY[project_setup.md]` for architecture decisions

2. **Understand the Codebase**:
   - Read relevant files using `view_file` or `view_code_item`
   - Check existing patterns and conventions
   - Identify similar implementations to maintain consistency

## 🔄 Post-Implementation Workflow

After implementing ANY new feature or change, you MUST complete ALL of the following steps:

### 1. Code Quality
```bash
# Run formatter
cargo fmt

# Run linter (fix auto-fixable issues)
cargo clippy --fix --allow-dirty

# Run linter (check remaining issues)
cargo clippy -- -D warnings
```

### 2. Testing
```bash
# Run all tests
cargo test

# Run integration tests specifically
cargo test --test integration_tests

# Run CLI black-box tests (if applicable)
.\tests\test_cli.ps1  # Windows
./tests/test_cli.sh   # Linux

# Run server tests (if server modified)
.\tests\test_server.ps1
```

### 3. Documentation Updates

**Code Documentation**:
- Add `///` doc comments to ALL public items (functions, structs, methods)
- Include `# Examples` section in doc comments
- Update module-level documentation if architecture changed

**Project Documentation**:
Update relevant files in `docs/`:
- `docs/architecture.md` - if architecture/design changed
- `docs/commands/*.md` - if CLI commands changed
- `docs/features/*.md` - if feature behavior changed
- `README.md` - if usage or installation changed

### 4. Version Control
- Update `task.md` artifact to mark items complete
- Update `walkthrough.md` artifact with summary of changes
- Ensure all changes are ready for commit

## 📋 Feature Implementation Checklist

For each new feature, ensure:

- [ ] Code follows `.agent/rules/rust_doc.md` standards
- [ ] All public APIs have documentation comments
- [ ] Unit tests added for new functionality
- [ ] Integration tests added if applicable
- [ ] CLI tests updated if commands changed
- [ ] `cargo fmt` run successfully
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes 100%
- [ ] Documentation updated (`docs/` folder)
- [ ] `task.md` and `walkthrough.md` artifacts updated

## 🎯 Quality Standards

### Code Quality
- **No Panics**: Use `Result<T, E>` instead of `.unwrap()` or `.expect()` in library code
- **Error Handling**: Use `anyhow` for applications, `thiserror` for libraries
- **Safety**: Never use `unsafe` without explicit `// SAFETY:` comment explaining why it's safe
- **Performance**: Profile before optimizing; trust zero-cost abstractions

### Testing Standards
- **Coverage**: Every public function must have tests
- **Integration**: Test real-world scenarios, not just units
- **Edge Cases**: Test error conditions, boundary values, empty inputs

### Documentation Standards
- **Public APIs**: Every `pub` item needs `///` documentation
- **Examples**: Include working code examples in documentation
- **Architecture**: Update `docs/architecture.md` when design changes
- **Commands**: Update `docs/commands/*.md` when CLI behavior changes

## 🚨 Common Pitfalls to Avoid

1. **Don't skip documentation** - It's not optional
2. **Don't commit without running tests** - Always verify `cargo test` passes
3. **Don't ignore clippy warnings** - Fix them before proceeding
4. **Don't modify public APIs** without updating docs and tests
5. **Don't use unwrap()** in library code - Return `Result` instead

## 📚 Reference Documentation

### Internal Documentation
- `.agent/rules/rust_doc.md` - Rust coding standards (MUST READ)
- `MEMORY[rust_bp.md]` - Comprehensive Rust best practices
- `MEMORY[project_setup.md]` - Project architecture and design decisions
- `docs/architecture.md` - System architecture overview

### External Resources
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [fastembed Documentation](https://docs.rs/fastembed/)
- [LanceDB Documentation](https://lancedb.github.io/lancedb/)

## 🔧 Development Workflow

### Standard Feature Implementation

1. **Plan** → Create `implementation_plan.md` artifact
2. **Research** → Read existing code, understand patterns
3. **Implement** → Write code following standards
4. **Test** → Add and run all tests
5. **Format** → Run `cargo fmt`
6. **Lint** → Run `cargo clippy --fix` then verify with `cargo clippy`
7. **Document** → Update code comments and `docs/`
8. **Verify** → Run full test suite
9. **Finalize** → Update `walkthrough.md` artifact

### Bug Fixes

1. **Reproduce** → Write failing test first
2. **Fix** → Implement minimal fix
3. **Verify** → Ensure test passes
4. **Regression** → Run full test suite
5. **Document** → Note fix in comments if non-obvious

## 🎬 Final Checklist Before Completion

Before marking any task complete:

- [ ] All tests passing (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Documentation updated (code comments + `docs/`)
- [ ] Artifacts updated (`task.md`, `walkthrough.md`)
- [ ] No `println!` debugging statements left in code
- [ ] No `TODO` or `FIXME` comments without associated task

---

**Remember**: Quality over speed. Take time to do it right the first time.
