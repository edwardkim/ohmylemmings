# Oh My Lemmings (Rusty Lemmings)

[한국어](README.md) | **English**

[![CI](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml/badge.svg)](https://github.com/edwardkim/ohmylemmings/actions/workflows/ci.yml)
[![Play on GitHub Pages](https://github.com/edwardkim/ohmylemmings/actions/workflows/pages.yml/badge.svg)](https://edwardkim.github.io/ohmylemmings/)

A noncommercial toy project reimplementing the data handling and behavior of the classic DOS game Lemmings in Rust. A deterministic game core is separated from rendering and I/O and shared by SDL2 native and WebAssembly web applications.

## Why this project is public and how it was built

This project was made public to explore reimplementation with an AI coding agent without using Reverse Engineer Anything (REA). Its purpose is to share an iterative process of implementation, testing, gameplay verification, and user feedback. Non-use of REA describes the development approach stated by the maintainer; not using a particular tool does not establish legal permission or independent authorship.

The public **Lemmix** implementation was used as a reference for physics, ticks, and state transitions, and **Lemmings.ts** for data parsing and web architecture. Results were compared against original game data and behavior. The OPL synthesizer uses and modifies an existing open-source implementation. Reference versions and provenance are recorded in the [reference code map](mydocs/tech_insights/issue-3_reference_code_map.md) and [pinned source inventory](assets/manifests/reference_sources.json).

Accordingly, **we do not claim that this was built without reverse engineering, without consulting the original or other implementations, or through a complete clean-room process.** It is a reimplementation informed by observation, data analysis, and public reference implementations.

## Licensing and the boundary around original assets

- Project-authored contributions use the [MIT License](LICENSE). Third-party code retains its own terms; see the [complete notices](THIRD_PARTY_NOTICES.txt) and [scope guide](third_party/README.md).
- The OPL synthesizer remains **LGPL-2.1-or-later**, the native Symphonia MP3 decoder **MPL-2.0**, and the Noto Sans font **OFL**. See the [OPL documentation](crates/lemmings-opl/README.md) and the packaged `REBUILD.md` for source, modification, and relinking instructions.
- **The project's MIT grant does not cover original DOS graphics, levels, music, sound effects, characters, or trademarks.** Redrawing assets in HD or generating them with AI does not, by itself, resolve rights in the original work.
- **The current v0.3.2 downloads and Pages deployment contain original game data.** The project materials reviewed do not establish permission to redistribute those original assets publicly. Provenance, hashes, and notice checks do not establish distribution rights.
- Research, educational, or noncommercial intent does not substitute for permission to use or redistribute original assets. This project does not claim to be an official product or to have the rights holder's endorsement or affiliation.
- User-selected local MP3 files are not uploaded or bundled. This feature does not remove copyright in those recordings or authorize every use.

This notice explains the project's purpose and the limits of what has been established. It is not an announcement that distribution rights have been secured or that original assets have been removed.

## Play and current status

As of 2026-10-25, the latest stable release is **v0.3.2**. Please also read the original-asset notice above.

- [Play in your browser](https://edwardkim.github.io/ohmylemmings/) · [Download for Windows/macOS/Linux/Web](https://github.com/edwardkim/ohmylemmings/releases/tag/v0.3.2)
- [v0.3.2 release and launch notes](mydocs/releases/issue-219_v0.3.2.md) · [Publication and public verification](mydocs/reports/issue-219_release_v032.md)
- Includes 120 original levels, eight skills, HD/original presentation switching, and improved terrain textures and vertical edges.
- Adds digging impact, dust and sound, a circular targeting marker, fatal-fall impact feedback, and disabled selection of skills with zero remaining uses.
- The web app downloads the HD assets needed for current play first, then loads upcoming level assets in the background. Progress, retries, and cache reuse are supported.
- Press F3 to show FPS and F2 to copy a feedback summary. [Performance and memory qualification #113](https://github.com/edwardkim/ohmylemmings/issues/113) remains open; 60 fps is not guaranteed.

Windows support is **Windows 11 x64**; Windows 10 is not supported. macOS requires **13+**, with an **Apple Silicon/Intel universal2** app that is ad-hoc signed and not notarized. Linux requires the SDL2 runtime. Extract the entire archive before launching.

Select local music with **MP3 선택** (MP3 selection) on the web, or **F4/Ctrl+O** on native platforms (**Cmd+O** is also supported on macOS), file drop, or `--bgm FILE`. Music and effects have separate volume controls. On the web, activate audio with **소리 켜기** (Enable sound).

## Development and running locally

[rust-toolchain.toml](rust-toolchain.toml) pins Rust **1.93.1**, rustfmt, and Clippy. Development requires GNU Make, Python 3.12, SDL2 development libraries, and the WASM target. Browser checks use Chrome/Chromium. See the [native guide](apps/desktop/README.md) and [web guide](apps/web/README.md) for platform setup.

```sh
rustup target add wasm32-unknown-unknown
cargo fetch --locked
python3 tools/web_build.py fetch
make check
```

Checks can also be run separately with `make check-rust`, `make check-python`, `make check-wasm`, `make check-desktop`, and `make check-web`. See the [CI guide](mydocs/plans/issue-7_continuous_integration.md) and [Makefile](Makefile).

The [asset guide](assets/README.md) describes preparation and identification of original data for users authorized to use it. A tool's ability to retrieve files is not permission to use or redistribute them. To run with locally prepared data:

```sh
cargo run --release --locked --offline -p lemmings-desktop -- --assets assets/local/dos-original
```

F/C/E/K/U/M/B/D select Floater/Climber/Bomber/Blocker/Builder/Miner/Basher/Digger. Select a skill, then click a lemming. Space pauses; up/down change the release rate; left/right move the camera; Esc exits. Platform guides cover additional controls, audio, and HD options.

## Structure and contributing

| Path | Responsibility |
|---|---|
| [lemmings-core](crates/lemmings-core/src/lib.rs) | `no_std` types, terrain, deterministic simulation |
| [lemmings-resources](crates/lemmings-resources/src/lib.rs) | `no_std` DOS resource parsing, images, level assembly |
| [lemmings-session](crates/lemmings-session/README.md) | Shared game session, timing, input |
| [lemmings-game](crates/lemmings-game/README.md) | Assets, scenes, panel, HD presentation, audio integration |
| [apps/desktop](apps/desktop/README.md) / [apps/web](apps/web/README.md) | SDL2 and JS/WASM platform adapters |
| `tools/` / `tools/tests/` | Acquisition, verification, packaging, and offline tests |

Read the [contribution guidelines](AGENTS.md) and [documentation guide](mydocs/README.md) first. The Hyper-Waterfall workflow starts with an issue and relevant documentation, followed by TDD, verification, self-review, a PR, and recorded results. Task documents use `mydocs/.../issue-<number>_<topic>.md`; daily journals use `mydocs/orders/yyyymmdd.md`. Downloaded binaries and private verification evidence belong in ignored `assets/local/`; provenance and hashes belong in `assets/manifests/`.

[Project plan](mydocs/plans/issue-1_rusty_lemmings.md) · [Implementation plan](mydocs/plans/issue-1_implementation_plan.md) · [Technical analysis](mydocs/tech_insights/README.md) · [Verification reports](mydocs/reports/README.md) · [Version/tag policy](mydocs/plans/issue-124_release_policy.md) · [Release history](mydocs/releases/README.md)

The [previous README's detailed development history](https://github.com/edwardkim/ohmylemmings/blob/a37aecd/README.md) is preserved as a historical record. Its “planned/next” statements do not describe the current implementation status. Linked development records are primarily in Korean; this English README covers the same current overview, purpose, licensing boundaries, and usage as the Korean README.
