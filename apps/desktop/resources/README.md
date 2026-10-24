# Application icon (#208)

Issue: https://github.com/edwardkim/ohmylemmings/issues/208

`app-icon.svg` is an original vector drawing by the project contributors, under the project's MIT license. It contains no downloaded artwork, fonts or original DOS pixels. The green-haired character and doorway follow the game's visual theme.

Regenerate with `python3 tools/app_icons.py generate` using `rsvg-convert` (librsvg). Renderer version and all source/output hashes are recorded in `assets/manifests/app_icon.json`; normal builds and offline checks use the committed outputs and do not need an SVG renderer.

- `app-icon.ico`: Windows PE group101, 16/24/32/48/64/128/256-pixel PNG representations. `apps/desktop/build.rs` embeds the icon and Cargo's version metadata using embed-resource; compilation failure is fatal.
- `app-icon.icns`: macOS bundle Resources and CFBundleIconFile, 16 through1024 pixels.
- `app-icon.png`: 256-pixel Linux/package icon.
- `app-icon.rgba`: 64×64 straight RGBA8 pixels embedded into SDL windows, independent of the working directory.

`python3 tools/app_icons.py verify` checks the inventory, PNG/ICO/ICNS structure and shared representations. Windows packaging reads the actual PE resource tree and compares all RT_ICON representations with the recorded artwork. No icon installation into the user's operating system is performed by the game.
