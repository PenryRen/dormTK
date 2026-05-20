#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/../clients/admin-desktop"
cargo run
