#!/bin/bash
set -euo pipefail
echo "=== OMNIVERSA Build ==="
rustc --version
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test --workspace
cargo bench
echo "=== Build Complete ==="
