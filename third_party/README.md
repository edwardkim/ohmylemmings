# Third-party notices

Issue: https://github.com/edwardkim/ohmylemmings/issues/129

The owner's MIT choice applies to project-authored contributions. Preserve these
upstream notices for references, adaptations and dependencies; they are not
replaced by the root MIT license. [THIRD_PARTY_NOTICES.txt](../THIRD_PARTY_NOTICES.txt)
contains each notice verbatim with its source URL and hash for binary/Web delivery.
[Inventory](../assets/manifests/license_inventory.json) pins the exact bytes and
the 18 locked Cargo dependency versions/checksums, including build dependencies.

- Lemmix uses its own project notice. Adapted Rust code is part of this
  reimplementation, not original Lemmix. The notice's references to DOS data and
  Graphics32 do not grant this project rights to those components.
- Lemmings.ts's complete notice includes MIT and a separate LGPL OPL appendix.
  Its JavaScript OPL emulator is not the product synthesizer.
- The product's Rust Nuked-OPL3 core and the C comparison oracle retain
  LGPL-2.1-or-later, original copyright, complete license and source/relink notices
  in [lemmings-opl](../crates/lemmings-opl/README.md) and
  [the C oracle](../tools/audio/reference/README.md).
- SDL2 runtime and the SDL bundled with sdl2-sys retain their Zlib notices.
  The sdl2-sys wrapper uses its repository root MIT notice, from the same pinned
  commit as sdl2. Cargo dependencies retain their full license alternatives;
  unicode-ident also retains Unicode-3.0.

Original DOS graphics, levels, music/effects and marks are outside the project
MIT grant. GAME-DATA records provenance/hashes, not a new license grant.
