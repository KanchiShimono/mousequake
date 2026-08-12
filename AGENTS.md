# Repository Guidelines

## Project Structure & Module Organization

Mousequake is a Rust 2024 CLI. `src/main.rs` defines the `clap` interface, shell completion, signal handling, and mouse loop. `src/trajectory.rs` contains the `Trajectory` trait and movement patterns. Unit tests live beside the code in each file's `#[cfg(test)]` module; there is no separate `tests/` directory or asset tree. CI and release automation are in `.github/workflows/`. Never commit Cargo's generated `target/` directory.

## Build, Test, and Development Commands

- `cargo build --locked`: compile a debug binary using the committed lockfile.
- `cargo run -- --help`: inspect the CLI without starting mouse movement.
- `cargo run -- -t circle -s 10 -i 5`: run a local example; stop it with Ctrl+C.
- `cargo test --workspace --locked --all-features --all-targets`: run all unit tests.
- `cargo fmt --all -- --check`: verify formatting exactly as CI does.
- `cargo clippy --workspace --locked --all-targets --all-features`: lint with warnings treated as errors.
- `cargo check --workspace --locked --all-targets --all-features`: perform the CI compile check quickly.

The stable toolchain includes `rustfmt` and `clippy` through `rust-toolchain.toml`.

## Coding Style & Naming Conventions

Use `rustfmt`, four-space indentation, and Unix newlines. Follow Rust conventions: `snake_case` for functions, modules, and tests; `PascalCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants. Keep CLI orchestration in `main.rs`; add reusable movement behavior to `trajectory.rs` behind `Trajectory`. Propagate recoverable errors with `Result` and `?`.

## Functional Programming Principles

Strictly adhere to these functional programming principles throughout development:

1. Parse, Don’t Validate
2. Make Illegal States Unrepresentable
3. Errors as values
4. Functional Core, Imperative Shell
5. Smart Constructor

## Idiomatic Paths

Apply these rules to all Rust code, including tests and `cfg`-gated code. Audit
inline paths containing `::` as well as `use` declarations.

- For free functions, import the parent module and call the function through
  it; for example, use `use std::fs;` and `fs::read_to_string(path)` instead of
  importing `read_to_string`. A function already qualified by its crate root
  or `super`, such as `super::run()`, needs no additional import.
- For structs, enums, traits, type aliases, and other non-function items,
  import the full item and use its short name everywhere, including fields,
  signatures, associated types, error variants, patterns, and expressions,
  unless the same-name rule applies. For example, use
  `use std::collections::HashMap;` and `HashMap<String, usize>`, not
  `std::collections::HashMap<String, usize>` inline.
- Qualify associated functions, constants, and variants through their imported
  owning type; for example, use `PathBuf::from(...)`, `usize::MAX`, and
  `Ordering::Less` rather than importing associated items separately.
- If a module provides both free functions and types, import the module for
  function calls and import each type directly; for example, use
  `use std::fs::{self, Metadata};`, then call `fs::metadata(path)` and refer to
  the return type as `Metadata`.
- For same-named items, prefer parent-module qualification; for example, use
  `use std::{fmt, io};` with `fmt::Result` and `io::Result`. Use an `as` alias
  only when it is a widely recognized Rust convention, such as
  `std::sync::atomic::Ordering as AtomicOrdering`, not merely to avoid
  qualification.

## Testing Guidelines

Add focused `#[test]` functions named `test_<behavior>` beside changed code. Trajectory tests should cover periodicity, displacement, and closed-path behavior with tolerances for floating-point comparisons. CLI changes should use `Cli::parse_from` so tests do not move the real pointer. No coverage threshold is configured, but every behavior change should include a regression test. CI tests Linux, macOS, and Windows.

## Commit & Pull Request Guidelines

Recent commits use short, imperative, sentence-case subjects, such as `Update README`; merged work commonly appends a PR number. Keep commits scoped and include `Cargo.lock` when dependencies change. Pull requests should explain the user-visible effect, list commands run, and link relevant issues. Include terminal output or completion examples for CLI changes; use screenshots only for platform-specific behavior. Ensure format, Clippy, check, and tests pass before review.

## Runtime Safety

Running the binary moves the pointer and may require OS accessibility or input permissions. Prefer unit tests during development; never commit credentials or machine-local settings.
