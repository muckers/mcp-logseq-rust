#!/usr/bin/env bash
# Build the release binary and ad-hoc sign it for macOS Sequoia.
# Usage: ./build-release.sh
set -e

cargo build --release
codesign -s "Local Dev Signing" --force --options runtime target/release/mcp-logseq-rust
echo "Build and signing complete: target/release/mcp-logseq-rust"
