#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pantheon-moira-worker.XXXXXX")
fixture_pid=
saga_pid=

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  if [ -n "$saga_pid" ] && kill -0 "$saga_pid" 2>/dev/null; then
    kill "$saga_pid" 2>/dev/null || true
    wait "$saga_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$repo/qualification/moira-participation/Cargo.toml" \
  --bin owner_worker_fixture --locked --offline
cargo build --manifest-path "$workspace/moira/Cargo.toml" \
  --package moira-agent-local --bin moira-agent-local --locked --offline
cargo build --manifest-path "$workspace/moira-tui/Cargo.toml" \
  --package moira-tui-cli --example real_daemon_fixture --locked --offline \
  --target-dir "$repo/qualification/moira-participation/target"
cargo build --manifest-path "$repo/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

fixture="$repo/qualification/moira-participation/target/debug/owner_worker_fixture"
worker="$workspace/moira/target/debug/moira-agent-local"
workbench="$repo/target/debug/pantheon"
saga_fixture="$repo/qualification/moira-participation/target/debug/examples/real_daemon_fixture"

start_saga() {
  label=$1
  rm -f "$scratch/saga.sock"
  "$saga_fixture" "$scratch/saga-$label.db" "$scratch/saga.sock" \
    >"$scratch/saga-$label.log" 2>&1 &
  saga_pid=$!
  attempt=0
  while [ ! -S "$scratch/saga.sock" ]; do
    if ! kill -0 "$saga_pid" 2>/dev/null || [ "$attempt" -eq 200 ]; then
      sed -n '1,200p' "$scratch/saga-$label.log" >&2
      echo "FAIL: Moira saga fixture $label did not become ready" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

finish_saga() {
  wait "$saga_pid"
  saga_pid=
}

printf '%s\n' \
  '{' \
  "  \"participation_socket\": \"$scratch/participation.sock\"," \
  "  \"daimon_agent_socket\": \"$scratch/daimon.sock\"," \
  '  "participation_id": "02020202020202020202020202020202",' \
  '  "output": {' \
  "    \"path\": \"$scratch/output.db\"," \
  '    "owner": "32323232323232323232323232323232",' \
  '    "max_entries": 4,' \
  '    "max_output_bytes": 4096,' \
  '    "max_page_assistant_bytes": 1024,' \
  '    "max_database_pages": 64' \
  '  },' \
  '  "policy": {' \
  '    "principal": "14141414141414141414141414141414",' \
  '    "workspace": "15151515151515151515151515151515",' \
  '    "context": ["16161616161616161616161616161616"],' \
  '    "tools": ["17171717171717171717171717171717"],' \
  '    "execution_budget": {' \
  '      "max_elapsed_millis": 1000,' \
  '      "max_input_tokens": 100,' \
  '      "max_output_tokens": 100,' \
  '      "max_tool_calls": 1,' \
  '      "max_context_bytes": 1024,' \
  '      "max_output_bytes": 1024' \
  '    },' \
  '    "session_budget": {' \
  '      "max_elapsed_millis": 10000,' \
  '      "max_turns": 4,' \
  '      "max_events": 32,' \
  '      "max_event_bytes": 4096,' \
  '      "max_prompt_bytes": 4096,' \
  '      "max_permission_requests": 4' \
  '    }' \
  '  },' \
  '  "transport": {' \
  '    "participation_max_frame_bytes": 65536,' \
  '    "participation_max_requests": 8,' \
  '    "daimon_handshake_max_frame_bytes": 1048576,' \
  '    "agent_max_frame_bytes": 1048576,' \
  '    "timeout_ms": 5000' \
  '  },' \
  '  "now": 7,' \
  '  "max_event_observations": 32' \
  '}' >"$scratch/agent.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Moira owner-worker qualification",' \
  '  "surfaces": ["moira"],' \
  '  "moira": {' \
  "    \"socket\": \"$scratch/saga.sock\"," \
  "    \"participation_socket\": \"$scratch/participation.sock\"," \
  '    "participation": "02020202020202020202020202020202",' \
  '    "agent_worker": {' \
  "      \"executable\": \"$worker\"," \
  "      \"config\": \"$scratch/agent.json\"" \
  '    }' \
  '  }' \
  '}' >"$scratch/profile.json"

"$fixture" serve "$scratch" >"$scratch/fixture.log" 2>&1 &
fixture_pid=$!
attempt=0
while [ ! -f "$scratch/ready" ]; do
  if ! kill -0 "$fixture_pid" 2>/dev/null || [ "$attempt" -eq 200 ]; then
    sed -n '1,200p' "$scratch/fixture.log" >&2
    echo 'FAIL: owner-worker fixture did not become ready' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

start_saga headless-before
before=$("$workbench" headless --workspace-profile "$scratch/profile.json")
finish_saga
printf '%s' "$before" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = next(value for value in report["peers"] if value["peer"] == "moira-participation")
assert peer["status"] == "current"
assert peer["phase"] == "Admitted" and peer["revision"] == 0
'

start_saga tui
python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary = sys.argv[1:]
root = Path(scratch)
tmux_socket = str(root / "tmux.sock")
session = "moira-owner-worker"
command = shlex.join([
    binary,
    "interactive",
    "--workspace-profile",
    str(root / "profile.json"),
]) + " 2>" + shlex.quote(str(root / "pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=5,
    )

def screen():
    captured = tmux("capture-pane", "-p", "-t", session, check=False)
    if captured.returncode != 0:
        error = (root / "pantheon.err").read_text(errors="replace")
        fixture = (root / "fixture.log").read_text(errors="replace")
        raise AssertionError(
            f"Pantheon exited before the expected screen:\n{error}\nFixture:\n{fixture}"
        )
    return captured.stdout

def expect(*markers, timeout=20):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    history = tmux("capture-pane", "-p", "-S", "-120", "-t", session).stdout
    error = (root / "pantheon.err").read_text(errors="replace")
    fixture = (root / "fixture.log").read_text(errors="replace")
    raise AssertionError(
        f"Missing {markers!r}:\n{history}\nPantheon:\n{error}\nFixture:\n{fixture}"
    )

def keys(*values):
    tmux("send-keys", "-t", session, *values)

def literal(value):
    tmux("send-keys", "-t", session, "-l", value)

def palette(query, marker):
    keys("C-p", "C-u")
    literal(query)
    expect(marker)
    keys("Enter")

try:
    tmux("new-session", "-d", "-x", "170", "-y", "36", "-s", session, command)
    expect("MOIRA", "Current governed revision 0", "phase: Admitted", timeout=30)

    palette("prepare selected moira agent session", "Prepare selected Moira agent session")
    expect("Moira prepare agent session", "c arms owner-derived open", "phase: Admitted")
    keys("c", "Enter")
    expect("Agent-session open durably prepared", "pending operation: Some(Open", timeout=20)

    palette("dispatch prepared moira agent session", "Dispatch prepared Moira agent session")
    expect("phase: Ready", "pending operation: none", timeout=20)
    keys("C-g")
    activity = expect("ACTIVITY & RECEIPTS", "Moira owner worker Observed participation revision 3")
    assert "OUTCOME UNKNOWN" not in activity
    keys("C-g", "C-q")

    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", session, check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after qualified owner-worker settlement")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
finish_saga

start_saga headless-after
after=$("$workbench" headless --workspace-profile "$scratch/profile.json")
finish_saga
printf '%s' "$after" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = next(value for value in report["peers"] if value["peer"] == "moira-participation")
assert peer["status"] == "current"
assert peer["phase"] == "Ready" and peer["revision"] == 3
assert peer["session_revision"] == 1
'

kill "$fixture_pid" 2>/dev/null || true
wait "$fixture_pid" 2>/dev/null || true
fixture_pid=
"$fixture" verify "$scratch"

echo 'PASS: Pantheon headless Admitted -> TUI prepared Open and dispatched real owner worker -> headless/durable Ready'
