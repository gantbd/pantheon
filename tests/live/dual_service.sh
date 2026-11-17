#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
vellum=$(CDPATH= cd -- "$repo/../vellum" && pwd -P)
chronos=$(CDPATH= cd -- "$repo/../chronos" && pwd -P)
# Keep both nested Unix socket names below macOS's sockaddr_un path bound even
# when TMPDIR expands to a long per-user directory.
scratch=$(mktemp -d /tmp/wb-live.XXXXXX)
chmod 700 "$scratch"
chronos_dir="$scratch/chronos"
vellum_socket="$scratch/vellum.sock"
vellum_control="$scratch/vellum-control.sock"
source_file="$scratch/source.txt"
chronos_target="$scratch/chronos-target"
vellum_target="$scratch/vellum-target"
workbench_target="$scratch/workbench-target"
chronos_bin="$chronos_target/debug/chronos-service"
vellum_bin="$vellum_target/debug/vellum-daemon"
workbench_bin="$workbench_target/debug/pantheon"
agora_bin="$workbench_target/debug/examples/agora_fixture_server"
agora_socket="$scratch/agora.sock"
chronos_log="$scratch/chronos.log"
vellum_log="$scratch/vellum.log"
agora_log="$scratch/agora.log"
chronos_pid=
vellum_pid=
agora_pid=

cleanup() {
  if [ -n "$agora_pid" ] && kill -0 "$agora_pid" 2>/dev/null; then
    kill "$agora_pid" 2>/dev/null || true
    wait "$agora_pid" 2>/dev/null || true
  fi
  if [ -n "$vellum_pid" ] && kill -0 "$vellum_pid" 2>/dev/null; then
    if [ -S "$vellum_control" ]; then
      "$vellum_bin" --shutdown "$vellum_control" >/dev/null 2>&1 || true
    fi
    wait "$vellum_pid" 2>/dev/null || true
  fi
  if [ -n "$chronos_pid" ] && kill -0 "$chronos_pid" 2>/dev/null; then
    "$chronos_bin" --shutdown "$chronos_dir" >/dev/null 2>&1 || true
    wait "$chronos_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

printf 'alpha' >"$source_file"
mkdir "$chronos_dir"
chmod 700 "$chronos_dir"
cargo build --manifest-path "$chronos/Cargo.toml" --package chronos-standalone \
  --bin chronos-service --locked --offline --target-dir "$chronos_target"
cargo build --manifest-path "$vellum/Cargo.toml" --package vellum-daemon \
  --bin vellum-daemon --locked --offline --target-dir "$vellum_target"
cargo build --manifest-path "$repo/Cargo.toml" --package pantheon-cli --bin pantheon \
  --example agora_fixture_server \
  --locked --offline --target-dir "$workbench_target"

start_agora() {
  rm -f "$agora_socket"
  "$agora_bin" "$agora_socket" >"$agora_log" 2>&1 &
  agora_pid=$!
  attempt=0
  while [ ! -S "$agora_socket" ]; do
    if ! kill -0 "$agora_pid" 2>/dev/null || [ "$attempt" -eq 100 ]; then
      sed -n '1,120p' "$agora_log" >&2
      echo 'FAIL: Agora fixture did not become ready' >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

finish_agora() {
  if ! wait "$agora_pid"; then
    sed -n '1,120p' "$agora_log" >&2
    echo 'FAIL: Agora fixture exited uncleanly' >&2
    exit 1
  fi
  agora_pid=
  rm -f "$agora_socket"
}

"$chronos_bin" "$chronos_dir" --atomic-state --observer-history --allow-writes >"$chronos_log" 2>&1 &
chronos_pid=$!
"$vellum_bin" --socket "$vellum_socket" --control "$vellum_control" \
  --allow-edit --allow-save -- "$source_file" >"$vellum_log" 2>&1 &
vellum_pid=$!

attempt=0
while [ "$attempt" -lt 100 ]; do
  if "$chronos_bin" --status "$chronos_dir" 2>/dev/null | grep -q 'ready=true' && \
     "$vellum_bin" --status "$vellum_control" 2>/dev/null | grep -q '"readiness":"ready"'; then
    break
  fi
  if ! kill -0 "$chronos_pid" 2>/dev/null || ! kill -0 "$vellum_pid" 2>/dev/null; then
    sed -n '1,120p' "$chronos_log" >&2
    sed -n '1,120p' "$vellum_log" >&2
    echo 'FAIL: a service exited before readiness' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.05
done
if [ "$attempt" -eq 100 ]; then
  sed -n '1,120p' "$chronos_log" >&2
  sed -n '1,120p' "$vellum_log" >&2
  echo 'FAIL: dual-service readiness timeout' >&2
  exit 1
fi

snapshot=$(
  "$workbench_bin" headless "$chronos_dir" --vellum-service "$vellum_socket"
)
printf '%s' "$snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["version"] == 1 and report["mode"] == "headless" and report["complete"]
peers = {peer["peer"]: peer for peer in report["peers"]}
assert peers["chronos"]["status"] == "current"
assert peers["vellum"]["status"] == "current"
'

profile="$scratch/chronos-profile.json"
printf '%s\n' '{"version":1,"workstreams":[{"name":"Chronos","directory":"chronos"},{"name":"Offline","directory":"offline"}]}' >"$profile"
set +e
snapshot=$("$workbench_bin" --chronos-profile "$profile" --vellum-service "$vellum_socket" --headless 2>/dev/null)
snapshot_status=$?
set -e
if [ "$snapshot_status" -eq 0 ]; then
  echo 'FAIL: partial headless workspace reported aggregate success' >&2
  exit 1
fi
printf '%s' "$snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert not report["complete"]
chronos = [peer for peer in report["peers"] if peer["peer"] == "chronos"]
assert [peer["status"] for peer in chronos] == ["current", "not_sent"]
assert next(peer for peer in report["peers"] if peer["peer"] == "vellum")["status"] == "current"
'

workspace_profile="$scratch/workspace-profile.json"
printf '%s\n' \
  '{"version":1,"name":"Live gate","surfaces":["overview","workspace","agora"],"agora":{"socket":"agora.sock","participant":"operator","requires":["rooms.read.v2"]},"chronos":{"workstreams":[{"name":"Chronos","directory":"chronos"},{"name":"Offline","directory":"offline"}],"moira":[],"daimon":[],"planning":null},"vellum":{"socket":"vellum.sock","credential":null}}' \
  >"$workspace_profile"
profile_check=$("$workbench_bin" --workspace-profile "$workspace_profile" --check-profile)
case "$profile_check" in
  'workspace profile valid: 3 surfaces'*) ;;
  *)
    echo 'FAIL: workbench did not validate the workspace profile' >&2
    exit 1
    ;;
esac

triple_profile="$scratch/triple-workspace.json"
printf '%s\n' \
  '{"version":1,"name":"Attached triple gate","surfaces":["overview","workspace","agora"],"agora":{"socket":"agora.sock","participant":"operator","requires":["rooms.read.v2"]},"chronos":{"workstreams":[{"name":"Chronos","directory":"chronos"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":{"socket":"vellum.sock","credential":null,"requires":["workspace.inventory.v1"]}}' \
  >"$triple_profile"
start_agora
triple_snapshot=$("$workbench_bin" headless --workspace-profile "$triple_profile")
finish_agora
printf '%s' "$triple_snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"] and report["mode"] == "headless"
assert [(peer["peer"], peer["status"]) for peer in report["peers"]] == [
    ("agora", "current"), ("chronos", "current"), ("vellum", "current")
]
'

# Exercise actual keyboard routing and focus with all three peer panes attached.
start_agora
python3 - "$scratch" "$workbench_bin" "$workspace_profile" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([binary, "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", socket, "-f", "/dev/null", *args],
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

def switch(name):
    tmux("send-keys", "-t", "workbench", "w")
    expect("Switch workstream")
    tmux("send-keys", "-t", "workbench", "-l", name)
    tmux("send-keys", "-t", "workbench", "Enter")

try:
    tmux("new-session", "-d", "-x", "240", "-y", "36", "-s", "workbench", command)
    expect("Chronos: attached", "Vellum: attached", "Agora: attached")
    tmux("send-keys", "-t", "workbench", "]")
    expect("Chronos  ·  revision", "Observed 0 tasks", "alpha")
    switch("Offline")
    expect("Offline  ·  revision", "Refresh failed", "alpha")
    switch("Chronos")
    expect("Chronos  ·  revision", "Observed 0 tasks", "alpha")
    tmux("send-keys", "-t", "workbench", "Tab")
    expect("focus=Vellum", "alpha")
    tmux("send-keys", "-t", "workbench", "Tab")
    expect("focus=Chronos")
    tmux("send-keys", "-t", "workbench", "]")
    expect("AGORA · rooms · revision 11", "Profile Room")
    tmux("send-keys", "-t", "workbench", "Enter")
    expect("Profile Agora transport is live")
    tmux("send-keys", "-t", "workbench", "[")
    expect("Chronos  ·  revision", "alpha")
    tmux("send-keys", "-t", "workbench", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "workbench", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
finish_agora

"$vellum_bin" --shutdown "$vellum_control" >/dev/null
wait "$vellum_pid"
vellum_pid=
"$chronos_bin" --shutdown "$chronos_dir" >/dev/null
wait "$chronos_pid"
chronos_pid=
echo 'PASS: attached Agora + Chronos + Vellum services -> one workbench composition'

hosted_profile="$scratch/hosted-workspace.json"
printf '%s\n' \
  '{"version":1,"name":"Mixed hosted gate","surfaces":["workspace","agora"],"agora":{"socket":"agora.sock","participant":"operator","requires":["rooms.read.v2"]},"chronos":{"deployment":"hosted","workstreams":[{"name":"Chronos","directory":"chronos"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":{"deployment":"hosted","socket":"vellum.sock","credential":null,"roots":["source.txt"],"requires":["workspace.inventory.v1"]}}' \
  >"$hosted_profile"

start_agora
hosted_snapshot=$("$workbench_bin" headless --workspace-profile "$hosted_profile")
finish_agora
printf '%s' "$hosted_snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"] and report["mode"] == "headless"
assert [(peer["peer"], peer["status"]) for peer in report["peers"]] == [
    ("agora", "current"), ("chronos", "current"), ("vellum", "current")
]
assert report["hosted_shutdown"]["status"] == "clean"
assert [receipt["peer"] for receipt in report["hosted_shutdown"]["receipts"]] == [
    "vellum", "chronos"
]
'

start_agora
python3 - "$scratch" "$workbench_bin" "$hosted_profile" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
socket = str(Path(scratch) / "hosted-tmux.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

try:
    tmux("new-session", "-d", "-x", "200", "-y", "32", "-s", "hosted", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "hosted").stdout
        if "Chronos  ·  revision" in screen and "alpha" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Pantheon panes not ready:\n{screen}")
    tmux("send-keys", "-t", "hosted", "]")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "hosted").stdout
        if "AGORA · rooms · revision 11" in screen and "Profile Room" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Attached Agora pane not ready:\n{screen}")
    tmux("send-keys", "-t", "hosted", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "hosted", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Hosted Pantheon did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
finish_agora

for endpoint in \
  "$chronos_dir/chronos.sock" \
  "$chronos_dir/chronos.control.sock" \
  "$vellum_socket" \
  "$vellum_socket.control"
do
  if [ -e "$endpoint" ]; then
    echo "FAIL: hosted shutdown retained endpoint $endpoint" >&2
    exit 1
  fi
done

echo 'PASS: hosted Chronos + Vellum with attached Agora -> headless and interactive Pantheon composition'
