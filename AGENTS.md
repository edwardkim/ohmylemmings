# Repository Guidelines

## Project Structure & Module Organization

This repository ports DOS Lemmings to Rust. `mydocs/plans/issue-1_rusty_lemmings.md` defines the architecture; `mydocs/plans/issue-1_implementation_plan.md` tracks incremental goals. G00 provides DOS reference tooling; Rust crates are planned.

When implementing:

- Separate resource parsing, deterministic simulation, and platform adapters into modules or crates. Keep simulation independent of rendering and I/O; prefer `no_std` compatibility where practical.
- Use `src/` for Rust code and `tests/` for integration tests within each crate.
- Use `mydocs/README.md` for document locations and naming rules.
- Keep acquisition tooling in `tools/` and its offline tests in `tools/tests/`.
- Store downloaded binaries in ignored `assets/local/`; commit provenance and hashes in `assets/manifests/`.

## Build, Test, and Development Commands

Run from the repository root with Python 3:

- `python3 tools/reference_assets.py fetch`: acquire and verify pinned reference files.
- `python3 tools/reference_assets.py verify`: check local file sizes and SHA-256 hashes.
- `python3 -m unittest discover -s tools/tests -v`: run offline tooling tests.

After G01 introduces `Cargo.toml`, use `cargo build`, `cargo test`, `cargo fmt --all -- --check`, and `cargo clippy --all-targets -- -D warnings`. Document SDL2 prerequisites and WASM commands alongside their adapters.

## Coding Style & Naming Conventions

Use four-space indentation; format Rust with `rustfmt`. Use `snake_case` for modules, files, and functions; `PascalCase` for types; and `SCREAMING_SNAKE_CASE` for constants. Record reference sources for simulation constants and behavior. Decouple logical ticks from rendering.

## Testing Guidelines

Use Python's `unittest` for tooling and Rust's built-in framework for future parser and simulation changes. Follow TDD for those Rust modules; keep unit tests nearby in `#[cfg(test)]` modules. Name tests by behavior, such as `truncated_dat_returns_error`.

Cover damaged or missing assets, malformed input, state transitions, collision boundaries, terrain changes, and repeatable ticks. Prefer synthetic fixtures; offline tests must work without original binaries. No coverage threshold is defined.

## Commit & Pull Request Guidelines

Use imperative commit subjects, for example `Add DAT section decoder`, and keep changes focused.

PRs should describe behavior, link issues, and report validation. Include screenshots for rendering/UI changes and reproducible fixtures for compatibility fixes.

## Hyper-Waterfall Development Workflow

Register a GitHub issue and verify its number/URL before writing task documents, editing files, or running experiments. Name `mydocs/` task documents `issue-<number>_<topic>.md` and link their issue. Keep folder `README.md` indexes. Use `orders/yyyymmdd.md` for Asia/Seoul daily journals with issue-numbered task lists. Read relevant documents, implement with TDD, record progress and validation, synchronize documentation, then close completed issues.
