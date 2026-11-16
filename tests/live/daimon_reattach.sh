#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-reattach.XXXXXX")
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
  -p daimon-tui-cli --bin daimon-tui --example agent_reattach_fixture --locked --offline
cargo build --quiet --manifest-path "$root/workbench-tui/Cargo.toml" \
  -p workbench-tui-cli --bin workbench-tui --locked --offline
fixture="$root/daimon-tui/target/debug/examples/agent_reattach_fixture"
standalone="$root/daimon-tui/target/debug/daimon-tui"
workbench="$root/workbench-tui/target/debug/workbench-tui"

printf '%s\n' \
  '{' \
  '  "schema": 1,' \
  '  "session": "07070707070707070707070707070707",' \
  '  "expected_revision": 12,' \
  '  "now": 34,' \
  '  "execution": {' \
  '    "execution_id": "0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a",' \
  '    "task_key": "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",' \
  '    "task_incarnation": 4,' \
  '    "attempt_number": 2,' \
  '    "principal": "0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c",' \
  '    "profile": "0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",' \
  '    "idempotency_key": "09090909090909090909090909090909",' \
  '    "context": ["0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e"],' \
  '    "tools": ["0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f"],' \
  '    "budget": {' \
  '      "max_elapsed_millis": 1000,' \
  '      "max_input_tokens": 100,' \
  '      "max_output_tokens": 200,' \
  '      "max_tool_calls": 3,' \
  '      "max_context_bytes": 4096,' \
  '      "max_output_bytes": 8192' \
  '    }' \
  '  }' \
  '}' >"$tmp/request.json"

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

start_fixture "$tmp/refused.sock" "$tmp/refused.marker"
if refused_output=$(
  "$standalone" agent-reattach "$tmp/refused.sock" "$tmp/request.json" reattach \
    2>"$tmp/refused.err"
); then
  echo 'wrong confirmation unexpectedly succeeded' >&2
  exit 1
fi
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$refused_output" | grep '"status": "confirmation_refused"' >/dev/null
test ! -e "$tmp/refused.marker"
confirmation=$(printf '%s\n' "$refused_output" | python3 -c 'import json,sys; print(json.load(sys.stdin)["required_confirmation"])')

start_fixture "$tmp/standalone.sock" "$tmp/standalone.marker"
standalone_output=$(
  "$standalone" agent-reattach "$tmp/standalone.sock" "$tmp/request.json" "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$standalone_output" | grep '"capability": "agents.session.reattach.v1"' >/dev/null
printf '%s\n' "$standalone_output" | grep '"status": "reattached"' >/dev/null
test "$(cat "$tmp/standalone.marker")" = reattached

start_fixture "$tmp/workbench.sock" "$tmp/workbench.marker"
workbench_output=$(
  "$workbench" daimon-agent-reattach "$tmp/workbench.sock" "$tmp/request.json" "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$workbench_output" | grep '"shell": "workbench"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"capability": "agents.session.reattach.v1"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"status": "reattached"' >/dev/null
test "$(cat "$tmp/workbench.marker")" = reattached

echo 'PASS: standalone and Workbench reuse one exact-detached Daimon reattach guard'
