# Check with clippy
clippy:
	cargo clippy --all

# Generate the documentation
doc:
	cargo doc --all

# Open the documentation in a browser
doc_open:
	cargo doc --package drone-core --open

# Generate README.md
readme:
	cargo readme -o README.md

# Run tests
test:
	cargo test --all --exclude drone-core
	cargo test --features std --package drone-core
