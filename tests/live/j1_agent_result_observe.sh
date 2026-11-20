#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /private/tmp/pj1result.XXXXXX)
fixture_pid=

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  if [ "${PANTHEON_KEEP_J1_RESULT_SCRATCH:-0}" = 1 ]; then
    echo "J1 result scratch retained for diagnosis: $scratch" >&2
  else
    rm -rf "$scratch"
  fi
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  --package daimon-tui-cli --example agent_open_journey_fixture --locked --offline
cargo build --manifest-path "$repo/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

fixture="$workspace/daimon-tui/target/debug/examples/agent_open_journey_fixture"
pantheon="$repo/target/debug/pantheon"
socket="$scratch/daimon.sock"
marker="$scratch/receipts"
session=17171717171717171717171717171717
turn=16161616161616161616161616161616

"$fixture" "$socket" "$marker" >"$scratch/fixture.out" 2>"$scratch/fixture.err" &
fixture_pid=$!
attempt=0
while [ ! -S "$socket" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$fixture_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/fixture.err" >&2
    echo 'FAIL: Daimon J1 result fixture did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

cat >"$scratch/open.json" <<EOF
{
  "schema": 1,
  "session": "$session",
  "workspace": "15151515151515151515151515151515",
  "now": 34,
  "execution": {
    "execution_id": "18181818181818181818181818181818",
    "task_key": "14141414141414141414141414141414",
    "task_incarnation": 1,
    "attempt_number": 1,
    "principal": "10101010101010101010101010101010",
    "profile": "11111111111111111111111111111111",
    "idempotency_key": "19191919191919191919191919191919",
    "context": ["12121212121212121212121212121212"],
    "tools": [],
    "budget": {
      "max_elapsed_millis": 60000,
      "max_input_tokens": 4096,
      "max_output_tokens": 8192,
      "max_tool_calls": 32,
      "max_context_bytes": 1048576,
      "max_output_bytes": 1048576
    }
  },
  "budget": {
    "max_elapsed_millis": 60000,
    "max_turns": 8,
    "max_events": 128,
    "max_event_bytes": 65536,
    "max_prompt_bytes": 16384,
    "max_permission_requests": 4
  }
}
EOF

set +e
"$pantheon" daimon-agent-open "$socket" "$scratch/open.json" wrong \
  >"$scratch/open-challenge.json" 2>"$scratch/open-challenge.err"
challenge_status=$?
set -e
test "$challenge_status" -ne 0
open_confirmation=$(jq -r .required_confirmation "$scratch/open-challenge.json")
test -n "$open_confirmation"
"$pantheon" daimon-agent-open "$socket" "$scratch/open.json" "$open_confirmation" \
  >"$scratch/opened.json"
jq -e '.capability == "agents.session.open.v1" and .status == "opened"' \
  "$scratch/opened.json" >/dev/null

printf '%s' 'Use the product-owner collaboration lens as an SME, not a human identity.
Work item:
local-agent-task' >"$scratch/prompt.txt"
cat >"$scratch/prompt.json" <<EOF
{
  "schema": 1,
  "session": "$session",
  "turn": "$turn",
  "expected_revision": 1,
  "now": 35,
  "execution": {
    "execution_id": "18181818181818181818181818181818",
    "task_key": "14141414141414141414141414141414",
    "task_incarnation": 1,
    "attempt_number": 1,
    "principal": "10101010101010101010101010101010",
    "profile": "11111111111111111111111111111111",
    "idempotency_key": "20202020202020202020202020202020",
    "context": ["12121212121212121212121212121212"],
    "tools": [],
    "budget": {
      "max_elapsed_millis": 60000,
      "max_input_tokens": 4096,
      "max_output_tokens": 8192,
      "max_tool_calls": 32,
      "max_context_bytes": 1048576,
      "max_output_bytes": 1048576
    }
  }
}
EOF

set +e
"$pantheon" daimon-agent-prompt "$socket" "$scratch/prompt.json" "$scratch/prompt.txt" wrong \
  >"$scratch/prompt-challenge.json" 2>"$scratch/prompt-challenge.err"
challenge_status=$?
set -e
test "$challenge_status" -ne 0
prompt_confirmation=$(jq -r .required_confirmation "$scratch/prompt-challenge.json")
test -n "$prompt_confirmation"
"$pantheon" daimon-agent-prompt "$socket" "$scratch/prompt.json" "$scratch/prompt.txt" \
  "$prompt_confirmation" >"$scratch/prompted.json"
jq -e '.capability == "agents.session.prompt.v1" and .status == "dispatched"' \
  "$scratch/prompted.json" >/dev/null

"$pantheon" j1-agent-result-observe "$socket" "$session" "$turn" \
  >"$scratch/result.json"
jq -e '
  .version == 1 and
  .peer == "daimon" and
  .capability == "agents.result.observe.v1" and
  .complete == true and
  .status == "qualified" and
  .session == "17171717171717171717171717171717" and
  .turn == "16161616161616161616161616161616" and
  .execution == "18181818181818181818181818181818" and
  .task.key == "14141414141414141414141414141414" and
  .task.incarnation == 1 and
  .task.attempt == 1 and
  .terminal_revision == 2 and
  .assistant_cursor == 1 and
  .finished_cursor == 2 and
  .output_digest == "sha256:56dca198aaaf18239d3cd5b36a4ae9bd486e7e6524aedf8ca94a17033b296870" and
  .content_bytes == 52 and
  .content == "The acceptance evidence is bounded and attributable." and
  (.boundary | contains("no Agora publication")) and
  (.boundary | contains("no Chronos mutation"))
' "$scratch/result.json" >/dev/null

test "$(cat "$marker")" = 'opened
prompted'

echo 'PASS: guarded J1 session open -> guarded prompt -> Daimon-owned execution/task-attempt result binding -> qualified read-only Pantheon observation'
