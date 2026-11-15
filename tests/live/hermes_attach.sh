#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
workspace=$(CDPATH= cd -- "$root/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-hermes.XXXXXX")
pid=
cleanup() { if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi; rm -rf "$tmp"; }
trap cleanup EXIT INT TERM
cargo run --quiet --manifest-path "$workspace/hermes-tui/Cargo.toml" -p hermes-tui-cli --example real_daemon_fixture -- "$tmp/ledger" "$tmp/hermes.sock" >"$tmp/out" 2>"$tmp/err" &
pid=$!
i=0
while [ ! -S "$tmp/hermes.sock" ]; do i=$((i+1)); if [ "$i" -ge 100 ]; then cat "$tmp/err" >&2; exit 1; fi; sleep 0.05; done
printf '%s\n' '{' '"version":1,' '"name":"Hermes attach qualification",' '"surfaces":["hermes"],' "\"hermes\":{\"socket\":\"$tmp/hermes.sock\",\"stream\":\"05050505050505050505050505050505\",\"consumer\":\"06060606060606060606060606060606\",\"requires\":[\"ledger.observe.v1\",\"ledger.cursor.observe.v1\"]}," '"agora":null,"moira":null,"daimon":null,"chronos":null,"vellum":null' '}' >"$tmp/profile.json"
output=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p workbench-tui-cli -- headless --workspace-profile "$tmp/profile.json")
wait "$pid"; pid=
printf '%s\n' "$output" | grep '"complete": true' >/dev/null
printf '%s\n' "$output" | grep '"peer": "hermes"' >/dev/null
printf '%s\n' "$output" | grep '"capability": "ledger.observe.v1"' >/dev/null
printf '%s\n' "$output" | grep '"streams": 1' >/dev/null
printf '%s\n' "$output" | grep '"retained_events": 1' >/dev/null
printf '%s\n' "$output" | grep '"configured_cursor": {' >/dev/null
printf '%s\n' "$output" | grep '"next_sequence": 1' >/dev/null
printf '%s\n' "$output" | grep '"backlog": 0' >/dev/null
