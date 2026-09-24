#!/usr/bin/env bash
# Set the crate version in Cargo.toml (and the matching Cargo.lock entry)
# without touching any dependency. Used by the release pipeline so builds
# carry the version computed from conventional commits.
#
# usage: packaging/set-version.sh 0.7.0
set -euo pipefail
VERSION="${1:?usage: set-version.sh <semver>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# only the [package] version line (first `version = "..."` in the file)
sed -i.bak -E "0,/^version = \"[^\"]+\"/s//version = \"$VERSION\"/" Cargo.toml && rm -f Cargo.toml.bak
# refresh the root package entry in Cargo.lock; -w (workspace only) leaves dependencies alone
cargo update --workspace --offline >/dev/null 2>&1 || cargo update --workspace >/dev/null
grep -m1 '^version' Cargo.toml
