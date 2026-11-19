#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
workspace=$(CDPATH= cd -- "$root/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pantheon-daimon-observe.XXXXXX")
fixture_pid=

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT INT TERM

cargo build --quiet --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  -p daimon-tui-cli --example agent_daemon_fixture --locked --offline
cargo build --quiet --manifest-path "$root/Cargo.toml" \
  -p pantheon-cli --bin pantheon --locked --offline

fixture="$workspace/daimon-tui/target/debug/examples/agent_daemon_fixture"
pantheon="$root/target/debug/pantheon"
socket="$scratch/agent.sock"

"$fixture" "$socket" >"$scratch/fixture.out" 2>"$scratch/fixture.err" &
fixture_pid=$!
attempt=0
while [ ! -S "$socket" ]; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 100 ]; then
    cat "$scratch/fixture.err" >&2
    exit 1
  fi
  sleep 0.05
done

"$pantheon" daimon-agent-observe "$socket" \
  07070707070707070707070707070707 >"$scratch/observation.json"
wait "$fixture_pid"
fixture_pid=

python3 - "$scratch/observation.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
raw = path.read_text()
value = json.loads(raw)
assert value["peer"] == "daimon"
assert value["capability"] == "agents.session.inspect.v1"
assert value["complete"] is True
assert value["status"] == "current"
assert value["observation"]["revision"] == 12
assert value["observation"]["phase"] == "ReattachRequired"
events = value["observation"]["events"]
assert [event["cursor"] for event in events] == [1, 2]
assert [event["kind"] for event in events] == ["plan", "usage"]
assert events[0]["evidence"]["bytes"] == 23
assert events[0]["evidence"]["content"] == "not_disclosed"
assert events[1]["evidence"] == {"input_tokens": 21, "output_tokens": 34}
assert "inspect before resuming" not in raw
assert "sha256" not in events[0]["evidence"]
assert "aggregate" not in value["boundary"]
PY
