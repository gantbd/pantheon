#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-batch.XXXXXX")
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

cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  --package daimon-tui-cli --example agent_set_fixture --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

fixture="$workspace/daimon-tui/target/debug/examples/agent_set_fixture"
workbench="$root/target/debug/pantheon"

"$fixture" "$scratch/daimon.sock" "$scratch/receipts" \
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
  "name": "Daimon bounded session-set qualification",
  "surfaces": ["daimon"],
  "daimon": {
    "agent_socket": "$scratch/daimon.sock",
    "agent_sessions": [
      "07070707070707070707070707070707",
      "08080808080808080808080808080808",
      "09090909090909090909090909090909"
    ],
    "requires": ["agents.session-set.inspect.v1", "agents.session-set.control.v1"]
  }
}
EOF

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "daimon-batch").stdout

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "daimon-batch", *values)

try:
    tmux("new-session", "-d", "-x", "160", "-y", "28", "-s", "daimon-batch", command)
    expect("session  1  Ready", "session  2  Ready", "session  3  Ready", timeout=30)
    keys("Space", "Right", "Space", "Right", "Space")
    expect("3 session(s) selected for bounded work")
    keys("C-p")
    keys("C-u")
    tmux("send-keys", "-t", "daimon-batch", "-l", "close selected daimon sessions")
    expect("Close selected Daimon sessions")
    keys("Enter")
    expect("Daimon guarded session set", "3 exact session controls")
    keys("c", "Enter")
    expect("Daimon run settled: 3 applied", "Closed r13", timeout=15)
    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "daimon-batch", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after the settled Daimon run")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

wait "$fixture_pid"
fixture_pid=
test "$(wc -l <"$scratch/receipts" | tr -d ' ')" = 3
grep 'session 1 closed' "$scratch/receipts" >/dev/null
grep 'session 2 closed' "$scratch/receipts" >/dev/null
grep 'session 3 closed' "$scratch/receipts" >/dev/null

echo 'PASS: configured 3-session workset -> sparse selection -> sequential owner guards -> three durable projections'
