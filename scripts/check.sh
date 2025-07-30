#!/bin/bash

set -euo pipefail

cargo fmt --check --all
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test
