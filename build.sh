#!/usr/bin/env bash
# Build the claude-code-wire WASM component (plugin.wasm).
#
# Requirements: Rust toolchain with the wasm32-unknown-unknown target
# (rustup target add wasm32-unknown-unknown).
#
# The guest is built for wasm32-unknown-unknown (NOT wasm32-wasip2): the
# freestanding std adds no WASI imports, so the resulting component imports
# only the declared memeloop host interface. The wit-bindgen component-type
# section is then wrapped into a component binary by the componentize helper
# (wit-component), with no WASI adapters.
set -euo pipefail
cd "$(dirname "$0")"

# Optionally refresh the vendored WIT files from a local
# memeloop-token-center checkout:  MTC_REPO=/path/to/memeloop-token-center ./build.sh
if [ -n "${MTC_REPO:-}" ]; then
  cp "$MTC_REPO/wit/wire-shim.wit" wit/wire-shim.wit
  cp "$MTC_REPO/wit/token-center.wit" wit/deps/token-center-0.2.0/token-center.wit
fi

cargo build --release --target wasm32-unknown-unknown
cargo run --release --manifest-path componentize/Cargo.toml --   target/wasm32-unknown-unknown/release/mtc_claude_code_wire.wasm   plugin.wasm
