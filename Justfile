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

# Bump crate versions
version-bump version drone-version libcore-drone-version:
	sed -i 's/\(docs\.rs\/drone-core\/\)[0-9]\+\(\.[0-9]\+\)\+/\1{{version}}/' \
		Cargo.toml src/lib.rs
	sed -i '/\[.*\]/h;/version = ".*"/{x;s/\[package\]/version = "{{version}}"/;t;x}' \
		Cargo.toml ctypes/Cargo.toml macros/Cargo.toml macros-core/Cargo.toml
	sed -i '/\[.*\]/h;/version = "=.*"/{x;s/\[.*drone-.*\]/version = "={{version}}"/;t;x}' \
		Cargo.toml macros/Cargo.toml
	sed -i '/\[.*\]/h;/version = ".*"/{x;s/\[.*drone-config\]/version = "{{drone-version}}"/;t;x}' \
		macros/Cargo.toml
	sed -i 's/\(drone-core.*\)version = "[^"]\+"/\1version = "{{version}}"/' \
		src/lib.rs
	sed -i 's/\(libcore-drone.*\)version = "[^"]\+"/\1version = "{{libcore-drone-version}}"/' \
		src/future/mod.rs

# Publish to crates.io
publish:
	cd ctypes && cargo publish
	sleep 5
	cd macros-core && cargo publish
	sleep 5
	cd macros && cargo publish
	sleep 5
	cargo publish
