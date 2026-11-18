#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-open.XXXXXX")
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
  -p daimon-tui-cli --bin daimon-tui --example agent_open_fixture --locked --offline
cargo build --quiet --manifest-path "$root/pantheon/Cargo.toml" \
  -p pantheon-cli --bin pantheon --locked --offline
fixture="$root/daimon-tui/target/debug/examples/agent_open_fixture"
standalone="$root/daimon-tui/target/debug/daimon-tui"
workbench="$root/pantheon/target/debug/pantheon"

printf '%s\n' \
  '{' \
  '  "schema": 1,' \
  '  "session": "07070707070707070707070707070707",' \
  '  "workspace": "10101010101010101010101010101010",' \
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
  '  },' \
  '  "budget": {' \
  '    "max_elapsed_millis": 60000,' \
  '    "max_turns": 8,' \
  '    "max_events": 128,' \
  '    "max_event_bytes": 65536,' \
  '    "max_prompt_bytes": 16384,' \
  '    "max_permission_requests": 4' \
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

start_fixture "$tmp/confirm.sock" "$tmp/confirm.marker"
if confirmation_output=$(
  "$standalone" agent-open "$tmp/confirm.sock" "$tmp/request.json" open \
    2>"$tmp/confirm.err"
); then
  echo 'wrong confirmation unexpectedly succeeded' >&2
  exit 1
fi
wait "$fixture_pid"
fixture_pid=
test ! -e "$tmp/confirm.marker"
confirmation=$(printf '%s\n' "$confirmation_output" | python3 -c 'import json,sys; print(json.load(sys.stdin)["required_confirmation"])')

start_fixture "$tmp/standalone.sock" "$tmp/standalone.marker"
standalone_output=$(
  "$standalone" agent-open "$tmp/standalone.sock" "$tmp/request.json" "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$standalone_output" | grep '"capability": "agents.session.open.v1"' >/dev/null
printf '%s\n' "$standalone_output" | grep '"status": "opened"' >/dev/null
test "$(cat "$tmp/standalone.marker")" = opened

start_fixture "$tmp/workbench.sock" "$tmp/workbench.marker"
workbench_output=$(
  "$workbench" daimon-agent-open "$tmp/workbench.sock" "$tmp/request.json" "$confirmation"
)
wait "$fixture_pid"
fixture_pid=
printf '%s\n' "$workbench_output" | grep '"shell": "workbench"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"capability": "agents.session.open.v1"' >/dev/null
printf '%s\n' "$workbench_output" | grep '"status": "opened"' >/dev/null
test "$(cat "$tmp/workbench.marker")" = opened

echo 'PASS: standalone and Pantheon reuse one exact Daimon session-open guard and request document'
