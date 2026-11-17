#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-moira-batch.XXXXXX")
aegis_pid=
owner_pid=

cleanup() {
  if [ -n "$owner_pid" ] && kill -0 "$owner_pid" 2>/dev/null; then
    kill -TERM "$owner_pid" 2>/dev/null || true
    wait "$owner_pid" 2>/dev/null || true
  fi
  if [ -n "$aegis_pid" ] && kill -0 "$aegis_pid" 2>/dev/null; then
    kill -TERM "$aegis_pid" 2>/dev/null || true
    wait "$aegis_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/aegis/Cargo.toml" \
  --package aegis-daemon --bin aegis-daemon --locked --offline
cargo build --manifest-path "$workspace/moira/Cargo.toml" \
  --package moira-saga-local --bin moira-saga-local \
  --example seed_uncertain_sagas --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

aegis="$workspace/aegis/target/debug/aegis-daemon"
owner="$workspace/moira/target/debug/moira-saga-local"
seed="$workspace/moira/target/debug/examples/seed_uncertain_sagas"
workbench="$root/target/debug/pantheon"
uid=$(id -u)
gid=$(id -g)

"$aegis" create "$scratch/authority" "$scratch/aegis.sock" \
  01010101010101010101010101010101 "$uid" "$gid" \
  02020202020202020202020202020202 0303030303030303 administrator \
  >"$scratch/aegis.out" 2>"$scratch/aegis.err" &
aegis_pid=$!

attempt=0
while [ ! -S "$scratch/aegis.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$aegis_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/aegis.err" >&2
    exit 1
  fi
  sleep 0.025
done

cat >"$scratch/owner.json" <<EOF
{
  "database": "$scratch/moira.db",
  "socket": "$scratch/moira.sock",
  "owner": "31313131313131313131313131313131",
  "mappings": [{
    "effective_uid": $uid,
    "effective_gid": $gid,
    "read": true,
    "mutate": true
  }],
  "io_timeout_ms": 3000,
  "max_requests_per_connection": 32,
  "max_frame_bytes": 65536,
  "max_checkpoint_bytes": 65536,
  "max_database_pages": 128,
  "max_sagas": 64,
  "max_retries_per_saga": 2,
  "max_inspection_page": 64,
  "max_operator_actions": 16,
  "operator": {
    "kind": "aegis",
    "socket": "$scratch/aegis.sock",
    "expected_epoch": 0,
    "grant_ttl_seconds": 300,
    "subject": "02020202020202020202020202020202",
    "sagas": [0, 1, 2, 3, 4, 5, 6, 7]
  }
}
EOF

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Moira batch qualification",
  "surfaces": ["moira"],
  "moira": {
    "socket": "$scratch/moira.sock",
    "requires": ["sagas.inspect.v1"]
  }
}
EOF

start_owner() {
  control=$1
  mkfifo "$control"
  exec 3<>"$control"
  "$owner" --supervised "$scratch/owner.json" <"$control" \
    >"$scratch/owner.out" 2>"$scratch/owner.err" &
  owner_pid=$!
  attempt=0
  while [ ! -S "$scratch/moira.sock" ]; do
    attempt=$((attempt + 1))
    if ! kill -0 "$owner_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
      sed -n '1,160p' "$scratch/owner.err" >&2
      exit 1
    fi
    sleep 0.025
  done
}

stop_owner() {
  printf x >&3
  exec 3>&-
  wait "$owner_pid"
  owner_pid=
  [ ! -e "$scratch/moira.sock" ]
}

start_owner "$scratch/control-first"
"$seed" "$scratch/moira.sock" 8 >"$scratch/seeded.json"

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux-batch.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "moira-batch").stdout

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "moira-batch", *values)

def text(value):
    tmux("send-keys", "-t", "moira-batch", "-l", value)

try:
    tmux("new-session", "-d", "-x", "160", "-y", "30", "-s", "moira-batch", command)
    expect("census total 8", "SagaIdV1(0)", timeout=30)
    keys("Space", "Down", "Space")
    expect("2 saga(s) selected for batch work")
    keys("C-p")
    text("keep selected moira")
    expect("Keep selected Moira sagas uncertain")
    keys("Enter")
    expect("Moira batch reconciliation", "2 sagas")
    expect("2 ready/not-sent")
    keys("c", "Enter")
    expect("Every reconciliation child durably applied", "2 durable")
    visible = screen()
    if visible.count("receipt") < 2:
        raise AssertionError(f"Per-child durable receipts were not visible:\n{visible}")
    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "moira-batch", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after settled batch")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

stop_owner
start_owner "$scratch/control-reopen"

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux-reopen.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", str(Path(scratch) / "profile.json")])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = tmux("capture-pane", "-p", "-t", "moira-reopen").stdout
        if all(marker in visible for marker in markers):
            return
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} after reopen:\n{visible}")

try:
    tmux("new-session", "-d", "-x", "150", "-y", "28", "-s", "moira-reopen", command)
    expect("census total 8", timeout=30)
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("operator receipts: 1", "authority", "KeepUncertain")
    tmux("send-keys", "-t", "moira-reopen", "BSpace")
    expect("census total 8", "SagaIdV1(1)")
    tmux("send-keys", "-t", "moira-reopen", "Down", "Enter")
    expect("saga: SagaIdV1(1)", "operator receipts: 1", "authority")
    tmux("send-keys", "-t", "moira-reopen", "BSpace", "Down", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "fail selected moira")
    expect("Fail selected Moira sagas terminally")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("Moira batch reconciliation", "1 sagas", "1 ready/not-sent")
    tmux("send-keys", "-t", "moira-reopen", "c", "Enter")
    expect("Every reconciliation child durably applied", "1 durable")
    tmux("send-keys", "-t", "moira-reopen", "Escape")
    expect("census total 8")
    tmux("send-keys", "-t", "moira-reopen", "Down")
    expect(">[ ] SagaIdV1(3)")
    tmux("send-keys", "-t", "moira-reopen", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "cancel selected moira sagas")
    expect("Cancel selected Moira sagas")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("logical time:", "This is Moira domain time")
    tmux("send-keys", "-t", "moira-reopen", "-l", "50")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("Moira batch reconciliation", "Cancel { now:", "1 ready/not-sent")
    tmux("send-keys", "-t", "moira-reopen", "c", "Enter")
    expect("Every reconciliation child durably applied", "1 durable")
    tmux("send-keys", "-t", "moira-reopen", "Escape")
    expect("census total 8")
    tmux("send-keys", "-t", "moira-reopen", "Down")
    expect(">[ ] SagaIdV1(4)")
    tmux("send-keys", "-t", "moira-reopen", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "compensate selected moira sagas")
    expect("Compensate selected Moira sagas")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("logical time:", "This is Moira domain time")
    tmux("send-keys", "-t", "moira-reopen", "-l", "50")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("Moira batch reconciliation", "Compensate { now:", "1 ready/not-sent")
    tmux("send-keys", "-t", "moira-reopen", "c", "Enter")
    expect("Every reconciliation child durably applied", "1 durable")
    tmux("send-keys", "-t", "moira-reopen", "Escape")
    expect("census total 8")
    tmux("send-keys", "-t", "moira-reopen", "Down", "Space", "Down", "Space")
    expect("2 saga(s) selected for batch work")
    tmux("send-keys", "-t", "moira-reopen", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "admit failed evidence selected moira")
    expect("Admit failed evidence for selected Moira sagas")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("logical time:", "current effect", "deadline 100")
    tmux("send-keys", "-t", "moira-reopen", "-l", "50")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("Moira batch reconciliation", "AdmitEvidence", "outcome: Failed", "2 ready/not-sent")
    tmux("send-keys", "-t", "moira-reopen", "c", "Enter")
    expect("Every reconciliation child durably applied", "2 durable")
    tmux("send-keys", "-t", "moira-reopen", "Escape")
    expect("census total 8")
    tmux("send-keys", "-t", "moira-reopen", "r")
    expect("SagaIdV1(5)  ForwardFailed", "SagaIdV1(6)  ForwardFailed")
    tmux("send-keys", "-t", "moira-reopen", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "retry failed effects selected moira")
    expect("Retry failed effects for selected Moira sagas")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("logical time:", "current effect", "deadline 100")
    tmux("send-keys", "-t", "moira-reopen", "-l", "60")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("Moira batch reconciliation", "Retry { failed_effect:", "2 ready/not-sent")
    tmux("send-keys", "-t", "moira-reopen", "c", "Enter")
    expect("Every reconciliation child durably applied", "2 durable")
    tmux("send-keys", "-t", "moira-reopen", "Escape")
    expect("census total 8")
    tmux("send-keys", "-t", "moira-reopen", "r")
    expect("SagaIdV1(5)  ForwardPending", "SagaIdV1(6)  ForwardPending")
    tmux("send-keys", "-t", "moira-reopen", "Space", "Up", "Space", "Down", "Down")
    expect(">[ ] SagaIdV1(7)")
    tmux("send-keys", "-t", "moira-reopen", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "admit successful evidence selected moira")
    expect("Admit successful evidence for selected Moira sagas")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("logical time:", "current effect", "deadline 100")
    tmux("send-keys", "-t", "moira-reopen", "-l", "50")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect("Moira batch reconciliation", "AdmitEvidence", "outcome: Succeeded", "1 ready/not-sent")
    tmux("send-keys", "-t", "moira-reopen", "c", "Enter")
    expect("Every reconciliation child durably applied", "1 durable")
    tmux("send-keys", "-t", "moira-reopen", "C-p")
    time.sleep(0.1)
    tmux("send-keys", "-t", "moira-reopen", "-l", "inspect tui instrumentation")
    expect("Inspect TUI instrumentation")
    tmux("send-keys", "-t", "moira-reopen", "Enter")
    expect(
        "TUI instrumentation",
        "0 in flight · 0 budget violations",
        "moira.reconciliation.fail-terminal",
        "moira.reconciliation.cancel",
        "moira.reconciliation.compensate",
        "moira.reconciliation.admit-failed",
        "moira.reconciliation.admit-succeeded",
        "moira.reconciliation.retry",
        "child_receipts",
    )
    tmux("send-keys", "-t", "moira-reopen", "Escape")
    tmux("send-keys", "-t", "moira-reopen", "C-q")
finally:
    tmux("kill-server", check=False)
PYTHON

stop_owner
start_owner "$scratch/control-final"

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux-final.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", str(Path(scratch) / "profile.json")])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = tmux("capture-pane", "-p", "-t", "moira-final").stdout
        if all(marker in visible for marker in markers):
            return
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} after terminal-failure reopen:\n{visible}")

try:
    tmux("new-session", "-d", "-x", "150", "-y", "28", "-s", "moira-final", command)
    expect("census total 8", timeout=30)
    tmux("send-keys", "-t", "moira-final", "Down", "Down", "Enter")
    expect("saga: SagaIdV1(2)", "phase: Failed", "operator receipts: 1", "FailTerminal")
    tmux("send-keys", "-t", "moira-final", "BSpace", "Down", "Enter")
    expect("saga: SagaIdV1(3)", "phase: CompensationPending", "operator receipts: 1", "Cancel")
    tmux("send-keys", "-t", "moira-final", "BSpace", "Down", "Enter")
    expect("saga: SagaIdV1(4)", "phase: CompensationPending", "operator receipts: 1", "Compensate")
    tmux("send-keys", "-t", "moira-final", "BSpace", "Down", "Enter")
    expect("saga: SagaIdV1(5)", "phase: ForwardPending", "operator receipts: 2", "AdmitEvidence", "Retry")
    tmux("send-keys", "-t", "moira-final", "BSpace", "Down", "Enter")
    expect("saga: SagaIdV1(6)", "phase: ForwardPending", "operator receipts: 2", "AdmitEvidence", "Retry")
    tmux("send-keys", "-t", "moira-final", "BSpace", "Down", "Enter")
    expect("saga: SagaIdV1(7)", "phase: ForwardSucceeded", "operator receipts: 1", "AdmitEvidence", "Succeeded")
    tmux("send-keys", "-t", "moira-final", "C-q")
finally:
    tmux("kill-server", check=False)
PYTHON

stop_owner
echo 'PASS: sparse N saga selection -> all reconciliation disposition classes -> exact effects/receipts -> restart-visible evidence'
