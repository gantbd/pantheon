#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-cancel.XXXXXX")
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
  -p daimon-tui-cli --bin daimon-tui --example agent_cancel_fixture --locked --offline
cargo build --quiet --manifest-path "$root/workbench-tui/Cargo.toml" \
  -p workbench-tui-cli --bin workbench-tui --locked --offline
fixture="$root/daimon-tui/target/debug/examples/agent_cancel_fixture"
standalone="$root/daimon-tui/target/debug/daimon-tui"
workbench="$root/workbench-tui/target/debug/workbench-tui"
session=07070707070707070707070707070707
turn=08080808080808080808080808080808
key=09090909090909090909090909090909
confirmation="CANCEL DAIMON SESSION $session TURN $turn REVISION 12 KEY $key AT 34"

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
  "$standalone" agent-cancel "$tmp/standalone.sock" "$session" "$turn" 12 "$key" 34 \
    "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$standalone_output" | grep '"capability": "agents.session.cancel.v1"' >/dev/null
printf '%s\n' "$standalone_output" | grep '"status": "cancelled"' >/dev/null
test "$(cat "$tmp/standalone.marker")" = cancelled

start_fixture "$tmp/workbench.sock" "$tmp/workbench.marker"
workbench_output=$(
  "$workbench" daimon-agent-cancel "$tmp/workbench.sock" "$session" "$turn" 12 "$key" 34 \
    "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$workbench_output" | grep '"shell": "workbench"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"capability": "agents.session.cancel.v1"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"status": "cancelled"' >/dev/null
test "$(cat "$tmp/workbench.marker")" = cancelled

echo 'PASS: standalone and Workbench reuse one exact-state Daimon cancellation guard'
