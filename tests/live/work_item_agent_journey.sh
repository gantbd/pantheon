#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
# Unix-domain endpoints have a small path ceiling on macOS; keep the complete
# Chronos/Daimon fixture topology under the short private temp root.
scratch=$(mktemp -d "/private/tmp/pwiaj.XXXXXX")
fixture_pid=

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/chronos"
chmod 700 "$scratch/chronos"

cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  --package daimon-tui-cli --example agent_set_fixture --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

fixture="$workspace/daimon-tui/target/debug/examples/agent_set_fixture"
workbench="$root/target/debug/pantheon"

"$fixture" "$scratch/daimon.sock" "$scratch/receipts" prompt \
  >"$scratch/fixture.out" 2>"$scratch/fixture.err" &
fixture_pid=$!
attempt=0
while [ ! -S "$scratch/daimon.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$fixture_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/fixture.err" >&2
    exit 1
  fi
  sleep 0.025
done

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Work item agent journey",
  "surfaces": ["chronos", "daimon"],
  "chronos": {
    "deployment": "hosted",
    "workstreams": [{"name": "Local work", "directory": "$scratch/chronos"}],
    "requires": ["tasks.observe.v1"],
    "moira": [],
    "daimon": [],
    "planning": null
  },
  "daimon": {
    "agent_socket": "$scratch/daimon.sock",
    "agent_sessions": ["07070707070707070707070707070707"],
    "execution": {
      "principal": "10101010101010101010101010101010",
      "profile": "11111111111111111111111111111111",
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
    "requires": ["agents.session-set.inspect.v1", "agents.session-set.prompt.v1"]
  }
}
EOF

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([
    binary, "interactive", "--workspace-profile", profile,
    "--chronos-allow-writes",
]) + " 2>" + shlex.quote(str(Path(scratch) / "pantheon.err"))
session = "work-item-agent"

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    captured = tmux("capture-pane", "-p", "-t", session, check=False)
    if captured.returncode != 0:
        error = Path(scratch, "pantheon.err").read_text(errors="replace")
        raise AssertionError(f"Pantheon exited before the expected screen:\n{error}")
    return captured.stdout

def expect(*markers, timeout=15):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", session, *values)

def literal(value):
    tmux("send-keys", "-t", session, "-l", value)

try:
    tmux("new-session", "-d", "-x", "170", "-y", "34", "-s", session, command)
    expect("CHRONOS", "Board", "revision 0", timeout=30)
    keys("C-p", "C-u")
    literal("create task with local agent")
    expect("Create task with local agent")
    keys("Enter")
    expect("Task batch")
    literal("inspect")
    keys("Tab")
    literal("report drift")
    keys("Enter")
    expect(
        "DAIMON",
        "Compose Daimon prompt",
        "inspect",
        "report drift",
        timeout=20,
    )
    keys("Tab", "c", "Enter")
    expect("Daimon guarded session set", "prompt ready session")
    keys("c", "Enter")
    expect(
        "Local-agent turn accepted",
        timeout=20,
    )
    keys("C-p", "C-u")
    literal("return to the kanban board")
    expect("Return to the Kanban board")
    keys("Enter")
    expect("CHRONOS", "Board", "inspect", "report drift", timeout=20)
    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", session, check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after the settled journey")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

kill "$fixture_pid" 2>/dev/null || true
wait "$fixture_pid" 2>/dev/null || true
fixture_pid=
test "$(wc -l <"$scratch/receipts" | tr -d ' ')" = 1
grep 'session 1 prompted' "$scratch/receipts" >/dev/null

echo 'PASS: Chronos task -> guarded local-agent prompt -> Daimon conversation -> refreshed Kanban board'
