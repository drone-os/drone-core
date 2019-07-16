# Check with clippy.
clippy:
	cargo clippy --all

# Generate documentation.
doc:
	cargo doc --all

# Generate README.md from src/lib.rs.
readme:
	cargo readme -o README.md

# Run tests.
test:
	cargo test --all --exclude drone-core
	cargo test --features "std" -p drone-core
