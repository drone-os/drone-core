# Install dependencies
deps:
	rustup component add clippy
	rustup component add rustfmt
	rustup component add rls rust-analysis rust-src
	type cargo-readme >/dev/null || cargo +stable install cargo-readme

# Reformat the source code
fmt:
	cargo fmt

# Check for mistakes
lint:
	cargo clippy --all

# Generate the docs
doc:
	cargo doc --all

# Open the docs in a browser
doc-open: doc
	cargo doc --package drone-core --open

# Run the tests
test:
	cargo test --all --exclude drone-core
	cargo test --features std --package drone-core

# Update README.md
readme:
	cargo readme -o README.md

# Publish to crates.io
publish:
	cd ctypes && cargo publish
	cd macros-core && cargo publish
	cd macros && cargo publish
	cargo publish
