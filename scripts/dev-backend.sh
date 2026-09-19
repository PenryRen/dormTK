#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/../env/dev"
podman-compose up -d --build
