#!/usr/bin/env bash

set -Eeuo pipefail

on_error() {
  local exit_code=$?
  echo "Cloudflare production build failed at line ${BASH_LINENO[0]} (exit ${exit_code})." >&2
  exit "${exit_code}"
}
trap on_error ERR

echo "Installing the minimal stable Rust toolchain..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

# rustup installs Cargo under the current build user's home directory.
# rustup creates this file during the build.
# shellcheck disable=SC1091
source "$HOME/.cargo/env"

echo "Adding the WebAssembly compilation target..."
rustup target add wasm32-unknown-unknown

echo "Installing pinned Trunk 0.21.14..."
cargo install trunk --version 0.21.14 --locked

echo "Building minified CSS..."
npm run css:build

echo "Building the production WebAssembly bundle..."
trunk build --release
