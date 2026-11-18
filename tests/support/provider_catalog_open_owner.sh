#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
  echo "hosted Daimon fixture requires COMMAND CONFIG" >&2
  exit 64
fi

case "$1" in
  --describe-providers)
    cat "$2"
    ;;
  --supervised)
    root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
    workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
    scratch=$(CDPATH= cd -- "$(dirname "$2")" && pwd -P)
    fixture="$workspace/daimon-tui/target/debug/examples/agent_open_journey_fixture"
    "$fixture" "$scratch/daimon.sock" "$scratch/receipts" \
      >"$scratch/fixture.out" 2>"$scratch/fixture.err" &
    fixture_pid=$!
    trap 'kill "$fixture_pid" 2>/dev/null || true; wait "$fixture_pid" 2>/dev/null || true; rm -f "$scratch/daimon.sock"' EXIT HUP INT TERM
    dd bs=1 count=1 >/dev/null 2>&1 || true
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
    rm -f "$scratch/daimon.sock"
    trap - EXIT HUP INT TERM
    ;;
  *)
    echo "hosted Daimon fixture supports --describe-providers or --supervised" >&2
    exit 64
    ;;
esac
