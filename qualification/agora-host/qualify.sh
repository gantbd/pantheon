#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
vendor="$workspace/chronos/third_party/keystone-storage-0.0.1"
shared="$workspace/keystone/crates/keystone-storage"
patch="patch.\"file://$vendor\".keystone-storage.path=\"$shared\""

exec cargo --config "$patch" test \
  --manifest-path "$repo/qualification/agora-host/Cargo.toml" \
  --all-targets --locked --offline "$@"
