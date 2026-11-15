#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
aegis_tui=$(CDPATH= cd -- "$repo/../aegis-tui" && pwd -P)
scratch=$(mktemp -d /tmp/wb-aegis.XXXXXX)
chmod 700 "$scratch"
target="$scratch/target"
fixture="$target/debug/examples/real_daemon_fixture"
workbench="$target/debug/workbench-tui"
socket="$scratch/aegis.sock"
authority="$scratch/authority"
profile="$scratch/workspace.json"
log="$scratch/aegis.log"
pid=

cleanup() {
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$aegis_tui/Cargo.toml" \
  --package aegis-tui-cli --example real_daemon_fixture \
  --locked --offline --target-dir "$target"
cargo build --manifest-path "$repo/Cargo.toml" \
  --package workbench-tui-cli --bin workbench-tui \
  --locked --offline --target-dir "$target"

cat >"$profile" <<'JSON'
{
  "version": 1,
  "name": "Aegis attach qualification",
  "surfaces": ["overview", "aegis"],
  "aegis": {
    "socket": "aegis.sock",
    "requires": ["authority.observe.v1"]
  }
}
JSON

start_fixture() {
  rm -f "$socket"
  rm -rf "$authority"
  "$fixture" "$authority" "$socket" >"$log" 2>&1 &
  pid=$!
  attempt=0
  while [ ! -S "$socket" ]; do
    if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 100 ]; then
      sed -n '1,120p' "$log" >&2
      echo 'FAIL: Aegis fixture did not become ready' >&2
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
assert peer["peer"] == "aegis"
assert peer["capability"] == "authority.observe.v1"
assert peer["status"] == "current" and peer["epoch"] == 0
assert peer["active_grants"] == 6 and peer["retained_decisions"] == 6
assert peer["page_grants"] == 4 and peer["more_grants"]
assert peer["page_decisions"] == 4 and peer["more_decisions"]
'

start_fixture
if refusal=$("$workbench" aegis-revoke-epoch "$socket" 0 'revoke aegis epoch 0' 2>"$scratch/revoke-confirmation.err"); then
  echo 'FAIL: Workbench accepted an inexact Aegis revocation confirmation' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refusal" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "aegis"
assert not value["complete"] and value["status"] == "confirmation_refused"
assert value["required_confirmation"] == "REVOKE AEGIS EPOCH 0"
'

start_fixture
if refusal=$("$workbench" aegis-revoke-epoch "$socket" 1 'REVOKE AEGIS EPOCH 1' 2>"$scratch/revoke-stale.err"); then
  echo 'FAIL: Workbench accepted a stale Aegis revocation epoch' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refusal" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert not value["complete"] and value["status"] == "stale_epoch"
'

start_fixture
revoked=$("$workbench" aegis-revoke-epoch "$socket" 0 'REVOKE AEGIS EPOCH 0')
finish_fixture
printf '%s' "$revoked" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "aegis"
assert value["complete"] and value["status"] == "accepted"
assert value["capability"] == "authority.revoke-epoch.v1"
assert value["expected_epoch"] == 0 and value["epoch"] == 1
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
    tmux("new-session", "-d", "-x", "120", "-y", "28", "-s", "workbench", command)
    expect("Aegis: attached")
    tmux("send-keys", "-t", "workbench", "]")
    expect("Aegis", "page: 1", "epoch: 0", "active grants: 6", "grant #0", "decision #3")
    tmux("send-keys", "-t", "workbench", "n")
    expect("page: 2", "grant #4", "grant #5", "decision #4", "decision #5")
    tmux("send-keys", "-t", "workbench", "p")
    expect("page: 1", "grant #0", "decision #0")
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

echo 'PASS: real Aegis daemon -> bounded pages and exact-confirmed revocation -> Workbench'
