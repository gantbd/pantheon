#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-close.XXXXXX")
fixture_pid=
cleanup() {
  if [ -n "$fixture_pid" ]; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

cargo build --quiet --manifest-path "$root/daimon-tui/Cargo.toml" \
  -p daimon-tui-cli --bin daimon-tui --example agent_close_fixture --locked --offline
cargo build --quiet --manifest-path "$root/workbench-tui/Cargo.toml" \
  -p workbench-tui-cli --bin workbench-tui --locked --offline
fixture="$root/daimon-tui/target/debug/examples/agent_close_fixture"
standalone="$root/daimon-tui/target/debug/daimon-tui"
workbench="$root/workbench-tui/target/debug/workbench-tui"
session=07070707070707070707070707070707
key=09090909090909090909090909090909
confirmation="CLOSE DAIMON SESSION $session REVISION 12 KEY $key AT 34"

start_fixture() {
  socket=$1
  marker=$2
  "$fixture" "$socket" "$marker" >"$socket.out" 2>"$socket.err" &
  fixture_pid=$!
  attempt=0
  while [ ! -S "$socket" ]; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 100 ]; then
      cat "$socket.err" >&2
      exit 1
    fi
    sleep 0.05
  done
}

start_fixture "$tmp/standalone.sock" "$tmp/standalone.marker"
standalone_output=$(
  "$standalone" agent-close "$tmp/standalone.sock" "$session" 12 "$key" 34 "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$standalone_output" | grep '"capability": "agents.session.close.v1"' >/dev/null
printf '%s\n' "$standalone_output" | grep '"status": "closed"' >/dev/null
test "$(cat "$tmp/standalone.marker")" = closed

start_fixture "$tmp/workbench.sock" "$tmp/workbench.marker"
workbench_output=$(
  "$workbench" daimon-agent-close "$tmp/workbench.sock" "$session" 12 "$key" 34 \
    "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$workbench_output" | grep '"shell": "workbench"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"capability": "agents.session.close.v1"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"status": "closed"' >/dev/null
test "$(cat "$tmp/workbench.marker")" = closed

echo 'PASS: standalone and Workbench reuse one exact-ready Daimon close guard'
