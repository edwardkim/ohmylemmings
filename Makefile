.PHONY: check check-rust check-python check-wasm check-desktop check-web web serve-web

check: check-rust check-python check-wasm check-desktop check-web

check-rust:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --locked --offline -- -D warnings
	cargo build --workspace --locked --offline
	cargo test --workspace --locked --offline
	python3 tools/check_state_replay.py
	python3 tools/check_adlib.py
	python3 tools/check_adlib_stream.py
	python3 tools/check_opl_reference.py
	python3 tools/check_audio.py
	python3 tools/check_game_sound.py

check-python:
	python3 -m unittest discover -s tools/tests -v
	python3 crates/lemmings-core/tests/fixtures/motion_reference.py

check-wasm:
	cargo build --workspace --lib --target wasm32-unknown-unknown --locked --offline

check-desktop:
	cargo build -p lemmings-desktop --release --locked --offline
	python3 tools/check_desktop.py --binary target/release/lemmings-desktop
	python3 tools/check_digger.py --binary target/release/lemmings-desktop
	python3 tools/check_basher.py --binary target/release/lemmings-desktop
	python3 tools/check_miner.py --binary target/release/lemmings-desktop
	python3 tools/check_builder.py --binary target/release/lemmings-desktop
	python3 tools/check_blocker.py --binary target/release/lemmings-desktop
	python3 tools/check_bomber.py --binary target/release/lemmings-desktop
	python3 tools/check_climber.py --binary target/release/lemmings-desktop
	python3 tools/check_floater.py --binary target/release/lemmings-desktop
	python3 tools/check_nuke.py --binary target/release/lemmings-desktop
	python3 tools/check_skill_interactions.py --binary target/release/lemmings-desktop
	python3 tools/check_exit.py --binary target/release/lemmings-desktop
	python3 tools/check_hazards.py --binary target/release/lemmings-desktop
	python3 tools/check_completion.py --binary target/release/lemmings-desktop
	python3 tools/check_results.py --binary target/release/lemmings-desktop
	python3 tools/check_panel.py --binary target/release/lemmings-desktop
	python3 tools/check_pointer.py --binary target/release/lemmings-desktop
	python3 tools/check_audio_output.py --binary target/release/lemmings-desktop

web:
	python3 tools/web_build.py build

serve-web:
	python3 tools/web_build.py serve

check-web:
	python3 tools/check_web.py
	python3 tools/check_web_journey.py
	python3 tools/check_audio.py --wasm
	python3 tools/check_game_sound.py --wasm
	python3 tools/check_audio_output.py --wasm
	python3 tools/sound_playthrough.py
