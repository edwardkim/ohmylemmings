# Repository Guidelines

## Project Structure & Module Organization

This repository is currently a planning-stage Rust port of DOS Lemmings. `mydocs/plans/rusty_lemmings.md` defines the architecture and milestones; no Cargo manifest, source tree, tests, or assets directory exists yet.

When implementing the plan:

- Separate resource parsing, deterministic simulation, and platform adapters into modules or crates. Keep simulation independent of rendering and I/O; prefer `no_std` compatibility where practical.
- Use `src/` for Rust code and `tests/` for integration tests within each crate.
- Use `mydocs/plans/`, `mydocs/orders/`, `mydocs/feedbacks/`, `mydocs/tech_insights/`, and `mydocs/troubleshootings/` for plans, tasks, feedback, reference findings, and troubleshooting.
- Document asset locations and setup when adding DAT/VGASPEC loading.

## Build, Test, and Development Commands

No build or run commands are available until a `Cargo.toml` is introduced. After Rust scaffolding, use these standard commands from the crate or workspace root:

- `cargo build`: compile the default targets.
- `cargo test`: run unit, integration, and documentation tests.
- `cargo fmt --all -- --check`: check Rust formatting.
- `cargo clippy --all-targets -- -D warnings`: lint configured targets and treat warnings as errors.

Document SDL2 prerequisites, executable names, and WASM build/run commands alongside the adapters when implemented.

## Coding Style & Naming Conventions

Use four-space indentation and standard `rustfmt` formatting. Use `snake_case` for modules, files, and functions; `PascalCase` for types and enum variants; and `SCREAMING_SNAKE_CASE` for constants. Name simulation constants by their meaning and record reference sources for original-game behavior. Keep logical ticks independent of rendering frequency.

## Testing Guidelines

Follow the plan's TDD workflow for parser and simulation changes, using Rust's built-in test framework. Keep unit tests near implementation in `#[cfg(test)]` modules. Use descriptive names such as `truncated_dat_returns_error` and `digger_removes_expected_pixels`.

Cover malformed binary input, state transitions, collision boundaries, terrain changes, and repeatable fixed-tick outcomes. Prefer small synthetic fixtures. No numeric coverage threshold is defined.

## Commit & Pull Request Guidelines

Use concise imperative commit subjects, for example `Add DAT section decoder`, and keep changes focused.

PRs should describe behavior, reference the relevant milestone or issue, and report checks run or unavailable. Include screenshots for rendering/UI changes and reproducible fixtures for compatibility fixes.

## Hyper-Waterfall Development Workflow

Follow the maintainer's Hyper-Waterfall methodology in `mydocs/plans/rusty_lemmings.md`: read relevant `mydocs/` documents before coding, implement small modules with TDD, and record findings, decisions, and troubleshooting results. Keep documentation synchronized with implementation and distinguish planned from completed work.
