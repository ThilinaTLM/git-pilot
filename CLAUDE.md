# git-tui

## Build & Test Commands
- `cargo build` — compile the project
- `cargo test` — run all tests (unit + integration)
- `cargo fmt` — format all code (run before committing)
- `cargo fmt --check` — check formatting without modifying
- `cargo clippy` — lint for common mistakes and style issues

## Code Quality
- Always run `cargo fmt` before committing
- All code must pass `cargo clippy` with no warnings
- All tests must pass before committing

## Architecture
- Layered: domain → infrastructure → app → ui
- Domain layer has no dependencies on other layers
- Infrastructure is trait-based for testability
- Single-threaded event loop with 250ms poll
