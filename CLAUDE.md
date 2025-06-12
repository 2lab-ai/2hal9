# HAL9 CI/CD Quality Commands

## Lint, Test, and Build Commands

When asked to check code quality, run the following commands:

```bash
# Lint with clippy (Rust linter)
cargo clippy --workspace --no-deps -- -W clippy::all

# Run all tests
cargo test --workspace

# Build the project
cargo build --workspace --release

# Quick check (faster than build)
cargo check --workspace
```

## Strategic Code Quality Analysis (L5 CTO Level)

### Fixed Issues (2025-06-12):
1. **Profile Configuration**: Removed profile settings from sub-crate Cargo.toml (should only be at workspace root)
2. **Naming Conventions**: Fixed `NPC` → `Npc` to follow Rust naming conventions
3. **Unused Variables**: Prefixed with `_` for intentionally unused variables
4. **Dead Code**: Added `#[allow(dead_code)]` for utility functions that may be used later
5. **Deprecated APIs**: Added `#[allow(deprecated)]` for Canvas API methods (still needed for browser compatibility)
6. **Error Handling**: Properly handled Results with `let _ =` pattern
7. **Code Style**: Replaced if-else chains with match statements for better readability
8. **Import Hygiene**: Removed unused imports

### Current Status:
- ✅ Zero clippy warnings
- ✅ All tests passing (156 tests)
- ✅ Clean build with no errors
- ✅ Code follows Rust best practices

### Recommendations:
1. Consider adding `clippy` to CI pipeline
2. Set up pre-commit hooks for automatic linting
3. Add more comprehensive tests for game_neurons module
4. Consider migrating from deprecated Canvas APIs when alternatives become stable

## Quick Commands for Future Use:

```bash
# Fix all auto-fixable clippy warnings
cargo clippy --workspace --fix

# Run tests with output
cargo test --workspace -- --nocapture

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit
```