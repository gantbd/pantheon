#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)

exec cargo test \
  --manifest-path "$repo/qualification/moira-participation/Cargo.toml" \
  --all-targets --locked --offline "$@"
