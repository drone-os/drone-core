# Check for mistakes
lint:
	rustup component add clippy
	cargo clippy --all

# Reformat the code
fmt:
	rustup component add rustfmt
	cargo fmt

# Generate the docs
doc:
	cargo doc --all

# Open the docs in a browser
doc_open: doc
	cargo doc --package drone-core --open

# Update README.md
readme:
	cargo readme -o README.md

# Run the tests
test:
	cargo test --all --exclude drone-core
	cargo test --features std --package drone-core
