cargo_features := '-Z features=itarget,build_dep,dev_dep -Z package-features'

# Install dependencies
deps:
	rustup component add clippy
	rustup component add rustfmt
	type cargo-readme >/dev/null || cargo +stable install cargo-readme

# Reformat the source code
fmt:
	cargo {{cargo_features}} fmt

# Check for mistakes
lint:
	cargo {{cargo_features}} clippy --all

# Generate the docs
doc:
	cargo {{cargo_features}} doc --all

# Open the docs in a browser
doc-open: doc
	cargo {{cargo_features}} doc --package drone-core --open

# Run the tests
test:
	cargo {{cargo_features}} test --all --exclude drone-core
	cargo {{cargo_features}} test --features std --package drone-core

# Update README.md
readme:
	cargo {{cargo_features}} readme -o README.md

# Bump crate versions
version-bump version drone-version:
	sed -i "s/\(api\.drone-os\.com\/drone-core\/\)[0-9]\+\(\.[0-9]\+\)\+/\1$(echo {{version}} | sed 's/\(.*\)\.[0-9]\+/\1/')/" \
		Cargo.toml ctypes/Cargo.toml macros/Cargo.toml macros-core/Cargo.toml src/lib.rs
	sed -i '/\[.*\]/h;/version = ".*"/{x;s/\[package\]/version = "{{version}}"/;t;x}' \
		Cargo.toml ctypes/Cargo.toml macros/Cargo.toml macros-core/Cargo.toml
	sed -i '/\[.*\]/h;/version = "=.*"/{x;s/\[.*drone-.*\]/version = "={{version}}"/;t;x}' \
		Cargo.toml macros/Cargo.toml
	sed -i '/\[.*\]/h;/version = ".*"/{x;s/\[.*drone-config\]/version = "{{drone-version}}"/;t;x}' \
		macros/Cargo.toml
	sed -i 's/\(drone-core.*\)version = "[^"]\+"/\1version = "{{version}}"/' \
		src/lib.rs

# Publish to crates.io
publish:
	cd ctypes && cargo {{cargo_features}} publish
	sleep 5
	cd macros-core && cargo {{cargo_features}} publish
	sleep 5
	cd macros && cargo {{cargo_features}} publish
	sleep 5
	cargo {{cargo_features}} publish

# Publish the docs to api.drone-os.com
publish-doc: doc
	dir=$(sed -n 's/.*api\.drone-os\.com\/\(.*\/.*\)\/.*\/"/\1/;T;p' Cargo.toml) \
		&& rm -rf ../drone-api/$dir \
		&& cp -rT target/doc ../drone-api/$dir \
		&& echo '<!DOCTYPE html><meta http-equiv="refresh" content="0; URL=./drone_core">' > ../drone-api/$dir/index.html \
		&& cd ../drone-api && git add $dir && git commit -m "Docs for $dir"
