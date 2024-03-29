all: test_all

build:
	cargo build --workspace

doc:
	cargo doc

open_doc:
	cargo doc --open

test_code: build
	cargo test --workspace --exclude matla_tests

test_top: build
	cargo run -p matla_tests -- --seq

test_all: build doc test_code test_top

test: test_all
