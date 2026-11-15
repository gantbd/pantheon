#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
workspace=$(CDPATH= cd -- "$root/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon.XXXXXX")
fixture_pid=
cleanup() {
  if [ -n "$fixture_pid" ]; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

cargo run --quiet --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  -p daimon-tui-cli --example real_daemon_fixture -- \
  "$tmp/store" "$tmp/daimon.sock" >"$tmp/daemon.out" 2>"$tmp/daemon.err" &
fixture_pid=$!

attempt=0
while [ ! -S "$tmp/daimon.sock" ]; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 100 ]; then
    cat "$tmp/daemon.err" >&2
    exit 1
  fi
  sleep 0.05
done

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Daimon attach qualification",' \
  '  "surfaces": ["daimon"],' \
  "  \"daimon\": {\"socket\": \"$tmp/daimon.sock\", \"requires\": [\"agents.presence.v1\"]}," \
  '  "agora": null,' \
  '  "moira": null,' \
  '  "chronos": null,' \
  '  "vellum": null' \
  '}' >"$tmp/profile.json"

output=$(cargo run --quiet --manifest-path "$root/Cargo.toml" \
  -p workbench-tui-cli -- headless --workspace-profile "$tmp/profile.json")
wait "$fixture_pid"
fixture_pid=

printf '%s\n' "$output" | grep '"complete": true' >/dev/null
printf '%s\n' "$output" | grep '"peer": "daimon"' >/dev/null
printf '%s\n' "$output" | grep '"capability": "agents.presence.v1"' >/dev/null
printf '%s\n' "$output" | grep '"agents": 1' >/dev/null
