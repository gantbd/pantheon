#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-agora-mutation.XXXXXX")
chmod 700 "$scratch"
target="$scratch/target"
socket="$scratch/agora.sock"
log="$scratch/daemon.log"
daemon_pid=''

cleanup() {
  if [ -n "$daemon_pid" ] && kill -0 "$daemon_pid" 2>/dev/null; then
    kill "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$workspace/agora-tui/Cargo.toml" \
  --package agora-tui-cli --example real_daemon_fixture \
  --locked --offline --target-dir "$target"
cargo build --manifest-path "$root/Cargo.toml" \
  --package workbench-tui-cli --bin workbench-tui \
  --locked --offline --target-dir "$target"

daemon="$target/debug/examples/real_daemon_fixture"
workbench="$target/debug/workbench-tui"

start_daemon() {
  rm -f "$socket"
  "$daemon" "$socket" >"$log" 2>&1 &
  daemon_pid=$!
  attempt=0
  while [ ! -S "$socket" ]; do
    if ! kill -0 "$daemon_pid" 2>/dev/null || [ "$attempt" -eq 200 ]; then
      sed -n '1,120p' "$log" >&2
      echo 'FAIL: real Agora daemon fixture did not become ready' >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

finish_daemon() {
  wait "$daemon_pid"
  daemon_pid=''
}

missing="$scratch/missing.sock"
if "$workbench" agora-create-room "$missing" operator conversation delivery-room \
  'Delivery Room' invite-only 3 'CREATE SOMETHING ELSE' \
  >"$scratch/confirmation.json" 2>"$scratch/confirmation.err"; then
  echo 'FAIL: Workbench accepted an inexact Agora confirmation' >&2
  exit 1
fi
python3 - "$scratch/confirmation.json" <<'PYTHON'
import json, sys
report = json.load(open(sys.argv[1]))
assert not report["complete"]
assert report["capability"] == "rooms.create.v2"
assert report["status"] == "confirmation_refused"
assert report["required_confirmation"] == (
    "CREATE AGORA ROOM delivery-room NAMED Delivery Room "
    "ADMISSION INVITE ONLY AT REVISION 3"
)
PYTHON
echo 'PASS: Workbench refused inexact Agora confirmation before connection'

start_daemon
if "$workbench" agora-create-room "$socket" operator conversation stale-room \
  'Stale Room' invite-only 2 \
  'CREATE AGORA ROOM stale-room NAMED Stale Room ADMISSION INVITE ONLY AT REVISION 2' \
  >"$scratch/stale.json" 2>"$scratch/stale.err"; then
  echo 'FAIL: Workbench accepted stale Agora room creation' >&2
  exit 1
fi
finish_daemon
python3 - "$scratch/stale.json" <<'PYTHON'
import json, sys
report = json.load(open(sys.argv[1]))
assert not report["complete"]
assert report["capability"] == "rooms.create.v2"
assert report["status"] == "refused"
PYTHON
echo 'PASS: real Agora daemon refused stale Workbench room creation'

start_daemon
created=$("$workbench" agora-create-room "$socket" operator conversation delivery-room \
  'Delivery Room' invite-only 3 \
  'CREATE AGORA ROOM delivery-room NAMED Delivery Room ADMISSION INVITE ONLY AT REVISION 3')
finish_daemon
printf '%s' "$created" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report == {
    "version": 1,
    "peer": "agora",
    "capability": "rooms.create.v2",
    "complete": True,
    "status": "created",
    "room_id": "delivery-room",
    "revision": 4,
}
'
echo 'PASS: real Agora daemon -> exact-confirmed Workbench room creation'
