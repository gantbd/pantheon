#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /tmp/wb-grounding.XXXXXX)
chmod 700 "$scratch"
target="$scratch/target"
workbench=${WORKBENCH_BIN:-"$target/debug/pantheon"}
profile="$scratch/workspace.json"
mixed_profile="$scratch/mixed-workspace.json"
pid=

cleanup() {
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

if [ -z "${WORKBENCH_BIN:-}" ]; then
  cargo build --manifest-path "$repo/Cargo.toml" \
    --package pantheon-cli --bin pantheon \
    --locked --offline --target-dir "$target"
fi

(cd "$workspace/dendrite" && \
  DENDRITE_GROUNDING_PORT_FILE="$scratch/port" MIX_ENV=test \
  mix run scripts/grounding_fixture.exs) >"$scratch/dendrite.log" 2>&1 &
pid=$!
attempt=0
while [ ! -s "$scratch/port" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 200 ]; then
    sed -n '1,160p' "$scratch/dendrite.log" >&2
    echo 'FAIL: Dendrite Grounding fixture did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done
endpoint="127.0.0.1:$(cat "$scratch/port")"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Grounding attach qualification",' \
  '  "surfaces": ["overview", "grounding"],' \
  "  \"grounding\": {\"endpoint\": \"$endpoint\", \"token\": null, \"requires\": [\"grounding.observe.v1\"]}" \
  '}' >"$profile"

snapshot=$("$workbench" headless --workspace-profile "$profile")
printf '%s' "$snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["version"] == 1 and report["mode"] == "headless" and report["complete"]
assert len(report["peers"]) == 1
peer = report["peers"][0]
assert peer["peer"] == "grounding" and peer["capability"] == "grounding.observe.v1"
assert peer["status"] == "current" and peer["entries"][0]["load_id"] == "fixture/sample"
'

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Grounding partial-failure qualification",' \
  '  "surfaces": ["overview", "aegis", "grounding"],' \
  "  \"aegis\": {\"socket\": \"$scratch/missing-aegis.sock\", \"requires\": [\"authority.observe.v1\"]}," \
  "  \"grounding\": {\"endpoint\": \"$endpoint\", \"token\": null, \"requires\": [\"grounding.observe.v1\"]}" \
  '}' >"$mixed_profile"

set +e
mixed_snapshot=$("$workbench" headless --workspace-profile "$mixed_profile" 2>"$scratch/mixed.err")
mixed_status=$?
set -e
if [ "$mixed_status" -eq 0 ]; then
  echo 'FAIL: partial peer observation reported aggregate success' >&2
  exit 1
fi
printf '%s' "$mixed_snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["version"] == 1 and report["mode"] == "headless"
assert not report["complete"]
assert [(peer["peer"], peer["status"]) for peer in report["peers"]] == [
    ("aegis", "connect_refused"),
    ("grounding", "current"),
]
assert report["peers"][1]["entries"][0]["load_id"] == "fixture/sample"
'

python3 - "$scratch" "$workbench" "$profile" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary, profile = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(["tmux", "-S", tmux_socket, "-f", "/dev/null", *args], capture_output=True, text=True, check=check, timeout=5)

def expect(*markers):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "workbench").stdout
        if all(marker in screen for marker in markers): return
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "120", "-y", "24", "-s", "workbench", command)
    expect("grounding", "current")
    tmux("send-keys", "-t", "workbench", "M-6")
    expect("Grounding", "fixture/sample", "current")
    tmux("send-keys", "-t", "workbench", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "workbench", check=False).returncode == 0:
        if time.monotonic() >= deadline: raise AssertionError("Pantheon did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

echo 'PASS: typed Dendrite Grounding endpoint -> Pantheon headless and interactive attach; healthy Grounding evidence survives failed Aegis peer'

