#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-moira-hosted.XXXXXX")
owner_pid=

cleanup() {
  if [ -n "$owner_pid" ] && kill -0 "$owner_pid" 2>/dev/null; then
    kill "$owner_pid" 2>/dev/null || true
    wait "$owner_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/moira/Cargo.toml" \
  --package moira-saga-local --bin moira-saga-local --example admit_saga \
  --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package workbench-tui-cli --bin workbench-tui --locked --offline

owner="$workspace/moira/target/debug/moira-saga-local"
admit="$workspace/moira/target/debug/examples/admit_saga"
workbench="$root/target/debug/workbench-tui"
socket="$scratch/moira.sock"
uid=$(id -u)
gid=$(id -g)

cat >"$scratch/owner.json" <<EOF
{
  "database": "$scratch/moira.db",
  "socket": "$socket",
  "owner": "31313131313131313131313131313131",
  "mappings": [{
    "effective_uid": $uid,
    "effective_gid": $gid,
    "read": true,
    "mutate": true
  }],
  "io_timeout_ms": 3000,
  "max_requests_per_connection": 16,
  "max_frame_bytes": 65536,
  "max_checkpoint_bytes": 65536,
  "max_database_pages": 128,
  "max_sagas": 64,
  "max_retries_per_saga": 2,
  "max_inspection_page": 64,
  "max_operator_actions": 16
}
EOF

mkfifo "$scratch/control"
exec 3<>"$scratch/control"
"$owner" --supervised "$scratch/owner.json" <"$scratch/control" \
  >"$scratch/owner.out" 2>"$scratch/owner.err" &
owner_pid=$!
attempt=0
while [ ! -S "$socket" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$owner_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/owner.err" >&2
    echo 'FAIL: standalone Moira owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done
[ "$("$admit" "$socket")" = 0 ]
printf x >&3
exec 3>&-
wait "$owner_pid"
owner_pid=
[ ! -e "$socket" ]

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Hosted Moira qualification",
  "surfaces": ["moira"],
  "moira": {
    "deployment": "hosted",
    "socket": "$socket",
    "requires": ["sagas.inspect.v1"],
    "host": {
      "executable": "$owner",
      "config": "$scratch/owner.json"
    }
  }
}
EOF

set +e
headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
headless_status=$?
set -e
if [ "$headless_status" -ne 0 ]; then
  printf '%s\n' "$headless" >&2
  echo 'FAIL: hosted Moira headless journey was incomplete' >&2
  exit "$headless_status"
fi
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = report["peers"][0]
assert peer["peer"] == "moira" and peer["status"] == "current"
assert peer["capability"] == "sagas.inspect.v1"
assert peer["sagas"] == 1 and peer["census"]["total"] == 1
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [{"peer": "moira", "instance": None, "disposition": "clean"}],
}
'
[ ! -e "$socket" ]

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

try:
    tmux("new-session", "-d", "-x", "130", "-y", "24", "-s", "moira", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "moira").stdout
        if "Moira" in screen and "census total 1" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Moira pane not ready:\n{screen}")
    tmux("send-keys", "-t", "moira", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "moira", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Workbench did not drain hosted Moira")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$socket" ]

set +e
reopened=$("$workbench" headless --workspace-profile "$scratch/profile.json")
reopened_status=$?
set -e
if [ "$reopened_status" -ne 0 ]; then
  printf '%s\n' "$reopened" >&2
  echo 'FAIL: hosted Moira reopen journey was incomplete' >&2
  exit "$reopened_status"
fi
printf '%s' "$reopened" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
assert report["peers"][0]["census"]["total"] == 1
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$socket" ]

echo 'PASS: production Moira owner -> public-client mutation -> Workbench hosted headless and interactive durable reopen -> clean drain'
