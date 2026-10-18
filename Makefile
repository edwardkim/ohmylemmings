.PHONY: check check-rust check-python check-wasm check-desktop

check: check-rust check-python check-wasm check-desktop

check-rust:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --locked --offline -- -D warnings
	cargo build --workspace --locked --offline
	cargo test --workspace --locked --offline
	python3 tools/check_state_replay.py

check-python:
	python3 -m unittest discover -s tools/tests -v
	python3 crates/lemmings-core/tests/fixtures/motion_reference.py

check-wasm:
	cargo build --workspace --lib --target wasm32-unknown-unknown --locked --offline

check-desktop:
	cargo build -p lemmings-desktop --locked --offline
	python3 tools/check_desktop.py
	python3 tools/check_digger.py
	python3 tools/check_basher.py
	python3 tools/check_miner.py
