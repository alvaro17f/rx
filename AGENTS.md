# AGENTS.md — Project Rules for `rx`

## Communication

- **CAVEMAN MODE FULL** always. Drop articles, filler, pleasantries. Short synonyms. Fragments fine. Technical terms exact. Code blocks unchanged. Pattern: thing action reason.
- Conversation in Spanish OK. Code, commits, docs in English.

## Workflow

- **TDD estricto**: write failing test first, then implement, then green. Always.
- **Coverage 100% strict**: regions, functions, lines — all must be 100%. Use `cargo llvm-cov --all-targets` to verify. No exceptions.
- **Never push without explicit permission**. Ask every time. No autonomous git push ever.
- Ask before destructive operations (purge, delete, force-push, rebase).

## Coverage

- Tool: `cargo llvm-cov --all-targets` exclusively. Rejected alternatives (tarpaulin).
- Nightly toolchain required (`#![feature(coverage_attribute)]` if `#[coverage(off)]` needed).
- **Prefer structural fixes** over `#[coverage(off)]`. Use `#[coverage(off)]` only as last resort for impossible-to-cover branches (e.g., std closure paths in `catch_unwind` helpers).
- **`dyn Write` over generics**: prefer `&mut dyn Write` instead of `<W: Write>` for IO functions. Generic monomorphizations create uncovered per-type regions.
- Every `?` creates a branch. Both paths (Ok, Err) must be exercised by tests.
- No `#[should_panic]` tests — they leave the normal return path uncovered. Use `std::panic::catch_unwind` in normal tests if panic-path coverage is needed.

## Code Style

- **No `.unwrap()` anywhere** — neither production nor test code.
- Use `.expect("context")` with descriptive messages in tests.
- Use `.is_ok()`, `.is_err()`, `.is_some()`, `.is_none()` for boolean assertions.
- Use `String::from_utf8_lossy(&buf)` instead of `String::from_utf8(buf).unwrap()`.
- Use `.expect_err("context")` instead of `.unwrap_err()`.
- **Rust best practices skill**: always apply `rust-best-practices` skill guidelines (borrowing over cloning, `?` over match chains, iterators over loops, descriptive test names, one assertion per test when possible, doc comments on public items).
- **SOLID / DRY / Clean Code**: extract functions for single responsibility. DRY repeated patterns. Keep functions small and readable.
- **Zero dependencies preference**: keep `Cargo.toml` dependency-free unless strong justification. Manual `Display`/`Error` impls over `thiserror` for binaries.

## Linting

- Run `cargo clippy --all-targets -- -D warnings` before considering work done. Zero warnings allowed.
- Use `#[expect(clippy::lint_name)]` with justification comment over `#[allow(...)]` if a lint must be suppressed.

## Commits

- **Conventional Commits** format: `type(scope): description`
- Types: `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `chore`, `ci`, `perf`, `build`, `revert`
- Imperative mood in description. Body optional for context.
- Small, focused commits preferred.
- Generate message from `git diff --cached` before committing.

## Testing

- Descriptive test names: `unit_should_return_x_when_y` or `function_returns_error_when_input_empty`.
- One assertion per test when possible.
- No `assert!`/`assert_eq!` macro overrides for coverage — use structural fixes instead.
- Test helpers (e.g., `ok`, `err`, `into_string`) must have both branches covered. Add `catch_unwind` tests for panic branches if needed.
- Integration tests in `tests/integration.rs` — exercise the compiled binary via `Command`.

## Architecture

- `Deps` trait for testability. `RealDeps` for production, `MockDeps` for unit tests.
- `dyn Write`/`dyn BufRead` for all IO functions — avoids monomorphization coverage gaps.
- `Error` enum: separate variants for distinct failure modes (e.g., `InvalidArgs` vs `GitPullFailed`).
- Extract phase functions from `cli()`: each phase (pull, stage, update, rebuild, cleanup, show_diff) has single responsibility.
- `write_parse_error()` for DRY error output in argument parsing.