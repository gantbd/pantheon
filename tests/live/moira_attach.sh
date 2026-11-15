#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
moira_tui=$(CDPATH= cd -- "$repo/../moira-tui" && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-moira.XXXXXX")
chmod 700 "$scratch"
target="$scratch/target"
fixture="$target/debug/examples/real_daemon_fixture"
workbench="$target/debug/workbench-tui"
socket="$scratch/moira.sock"
database="$scratch/moira.db"
profile="$scratch/workspace.json"
log="$scratch/moira.log"
pid=

cleanup() {
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$moira_tui/Cargo.toml" \
  --package moira-tui-cli --example real_daemon_fixture \
  --locked --offline --target-dir "$target"
cargo build --manifest-path "$repo/Cargo.toml" \
  --package workbench-tui-cli --bin workbench-tui \
  --locked --offline --target-dir "$target"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Moira attach qualification",' \
  '  "surfaces": ["moira"],' \
  "  \"moira\": {\"socket\": \"$socket\", \"requires\": [\"sagas.inspect.v1\"]}" \
  '}' >"$profile"

start_fixture() {
  rm -f "$socket" "$database"
  "$fixture" "$database" "$socket" >"$log" 2>&1 &
  pid=$!
  attempt=0
  while [ ! -S "$socket" ]; do
    if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 100 ]; then
      sed -n '1,160p' "$log" >&2
      echo 'FAIL: Moira fixture did not become ready' >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

finish_fixture() {
  wait "$pid"
  pid=
}

start_fixture
snapshot=$("$workbench" headless --workspace-profile "$profile")
finish_fixture
printf '%s' "$snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["version"] == 1 and report["mode"] == "headless" and report["complete"]
assert len(report["peers"]) == 1
peer = report["peers"][0]
assert peer["peer"] == "moira"
assert peer["capability"] == "sagas.inspect.v1"
assert peer["status"] == "current" and peer["sagas"] == 1
assert peer["census"]["total"] == 1 and peer["census"]["fenced"] == 1
'

start_fixture
python3 - "$scratch" "$workbench" "$profile" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def expect(*markers):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "workbench").stdout
        if all(marker in screen for marker in markers):
            return
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "140", "-y", "32", "-s", "workbench", command)
    expect("Moira", "census total 1", "FencedUncertain")
    tmux("send-keys", "-t", "workbench", "Enter")
    expect("evidence page: 1", "operator receipts: 1", "authority 9/10")
    tmux("send-keys", "-t", "workbench", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "workbench", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Workbench did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
finish_fixture

echo 'PASS: real Moira daemon -> census plus selected-saga evidence and operator receipts -> Workbench headless and interactive attach'
