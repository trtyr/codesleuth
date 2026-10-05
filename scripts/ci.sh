#!/usr/bin/env bash
# 三连全绿门（baseline/test-and-release-gates.md）
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --check
cargo clippy -- -D warnings
cargo test
