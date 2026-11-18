#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)

cargo test \
  --manifest-path "$repo/qualification/moira-participation/Cargo.toml" \
  --all-targets --locked --offline "$@"

exec "$repo/qualification/moira-participation/live_owner_worker.sh"
