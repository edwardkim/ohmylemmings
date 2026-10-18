# Repository Guidelines

## Project Structure & Module Organization

This repository ports DOS Lemmings to Rust. `crates/lemmings-core` owns domain types; `crates/lemmings-resources` handles borrowed byte input. Both are `no_std`. `mydocs/plans/` defines architecture; platform adapters follow in G06/G09.

When implementing:

- Keep parsing, simulation, and platform adapters separate. Simulation must avoid rendering and I/O dependencies.
- Keep Rust sources in each crate's `src/` and integration tests in `tests/`.
- Use `mydocs/README.md` for document locations and naming rules.
- Keep acquisition tooling in `tools/` and its offline tests in `tools/tests/`.
- Store downloaded binaries in ignored `assets/local/`; commit provenance and hashes in `assets/manifests/`.

## Build, Test, and Development Commands

Run at the repository root; `rust-toolchain.toml` pins Rust 1.93.1:

- `make check`: run all CI checks locally.
- `make check-rust`: build, test, format, and lint both libraries offline.
- `make check-python`: run tooling tests.
- `make check-wasm`: compile WASM libraries; install the target first.

Python 3 reference tooling:

- `python3 tools/reference_assets.py fetch`: acquire and verify pinned reference files.
- `python3 tools/reference_assets.py verify`: check local file sizes and SHA-256 hashes.
- `python3 -m unittest discover -s tools/tests -v`: run offline tooling tests.

See `mydocs/plans/issue-7_continuous_integration.md` for prerequisites and CI details. Document SDL2 prerequisites with platform adapters.

## Coding Style & Naming Conventions

Use four-space indentation; format Rust with `rustfmt`. Use `snake_case` for modules, files, and functions; `PascalCase` for types; and `SCREAMING_SNAKE_CASE` for constants. Record reference sources for simulation constants and behavior. Decouple logical ticks from rendering.

## Testing Guidelines

Use Python's `unittest` and Rust's built-in framework. Follow TDD and keep unit tests in nearby `#[cfg(test)]` modules. Name tests by behavior, such as `truncated_dat_returns_error`.

Cover damaged or missing assets, malformed input, state transitions, collision boundaries, terrain changes, and repeatable ticks. Prefer synthetic fixtures; offline tests must work without original binaries. No coverage threshold.

## Commit & Pull Request Guidelines

Use imperative commit subjects, for example `Add DAT section decoder`, and keep changes focused.

PRs should describe behavior, link issues, and report validation. Include screenshots for rendering/UI changes and reproducible fixtures for compatibility fixes.

## Hyper-Waterfall Development Workflow

Register a GitHub issue and verify its number/URL before writing task documents, editing files, or running experiments. Name `mydocs/` task documents `issue-<number>_<topic>.md` and link their issue. Keep folder `README.md` indexes. Use `orders/yyyymmdd.md` for Asia/Seoul daily journals with issue-numbered task lists. Read relevant documents, implement with TDD, record validation, synchronize documentation, then close completed issues.
