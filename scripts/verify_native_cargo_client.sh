#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
index="${TEAQL_CARGO_NATIVE_INDEX:-}"
if [[ "${TEAQL_CARGO_NATIVE_ISOLATED:-}" != "1" ]]; then
  echo 'TEAQL_CARGO_NATIVE_ISOLATED=1 is required for disposable package publication' >&2
  exit 1
fi
if [[ ! "$index" =~ ^sparse\+http://(127\.0\.0\.1|localhost):[0-9]+/repository/[^/]+/cargo/index/$ ]]; then
  echo 'TEAQL_CARGO_NATIVE_INDEX must be a localhost-only sparse Cargo index ending in /cargo/index/' >&2
  exit 1
fi
if [[ -z "${CARGO_REGISTRIES_TEAQL_TOKEN:-}" ]]; then
  echo 'CARGO_REGISTRIES_TEAQL_TOKEN is required; use a test-only token for the isolated registry' >&2
  exit 1
fi

gate_dir="$(mktemp -d /tmp/teaql-native-cargo-gate.XXXXXX)"
trap 'rm -rf -- "$gate_dir"' EXIT
mkdir -p "$gate_dir/base" "$gate_dir/child" "$gate_dir/consumer"
cp -R "$repo_dir/examples/cargo-registry-native/base/." "$gate_dir/base/"
cp -R "$repo_dir/examples/cargo-registry-native/child/." "$gate_dir/child/"
cp -R "$repo_dir/examples/cargo-registry-native/consumer/." "$gate_dir/consumer/"
export CARGO_REGISTRIES_TEAQL_INDEX="$index"
export CARGO_TARGET_DIR="$gate_dir/target"
export CARGO_HOME="$gate_dir/publish-home"

cargo publish --registry teaql --manifest-path "$gate_dir/base/Cargo.toml"
cargo publish --registry teaql --manifest-path "$gate_dir/child/Cargo.toml"

export CARGO_HOME="$gate_dir/consumer-home"
export CARGO_TARGET_DIR="$gate_dir/consumer-target"
cargo run --manifest-path "$gate_dir/consumer/Cargo.toml"
cargo run --offline --locked --manifest-path "$gate_dir/consumer/Cargo.toml"

lock="$gate_dir/consumer/Cargo.lock"
if [[ "$(grep -Fc "source = \"$index\"" "$lock")" != "2" ]]; then
  echo 'FAIL: consumer Cargo.lock does not retain two packages from the specified index' >&2
  exit 1
fi
echo 'PASS native Cargo transitive dependency gate (fresh consumer and offline locked replay)'
