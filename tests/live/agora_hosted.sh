#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-agora-hosted.XXXXXX")
aegis_pid=''
cleanup() {
  if [ -n "$aegis_pid" ] && kill -0 "$aegis_pid" 2>/dev/null; then
    kill -TERM "$aegis_pid" 2>/dev/null || true
    wait "$aegis_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

if [ "${WORKBENCH_SKIP_BUILD:-0}" != 1 ]; then
  cargo build --manifest-path "$workspace/aegis/Cargo.toml" \
    --package aegis-daemon --bin aegis-daemon --locked --offline
  cargo build --manifest-path "$workspace/agora/Cargo.toml" \
    --package agora-private-local --bin agora-private-local --locked --offline
  cargo build --manifest-path "$root/Cargo.toml" \
    --package workbench-tui-cli --bin workbench-tui --locked --offline
fi

aegis="$workspace/aegis/target/debug/aegis-daemon"
agora="$workspace/agora/target/debug/agora-private-local"
workbench="$root/target/debug/workbench-tui"
uid=$(id -u)
gid=$(id -g)

run_profile() {
  mode=$1
  epoch=$2
  incarnation=$3
  operation=${4:-headless}
  grant_now=$(( $(date +%s) * 1000 ))
  grant_expires_at=$(( grant_now + 600000 ))
  config="$scratch/agora-$mode.conf"
  profile="$scratch/$mode.json"

  if [ "$mode" = create ]; then
    printf '%s\n' \
      'schema=1' \
      'mode=create' \
      "store=$scratch/agora-store" \
      "socket=$scratch/agora.sock" \
      "aegis_socket=$scratch/aegis.sock" \
      'participant=alice' \
      'subject=02020202020202020202020202020202' \
      'resource=07070707070707070707070707070707' \
      "authority_epoch=$epoch" \
      "grant_now=$grant_now" \
      "grant_expires_at=$grant_expires_at" \
      'grant_profile=scheduled' \
      'host_scope=08080808080808080808080808080808' \
      "server_incarnation=$incarnation" \
      'display_name=Alice' \
      'participant_kind=human' \
      'room_id=scheduled-room' \
      'room_name=Scheduled Room' \
      'admission=open' \
      >"$config"
  else
    printf '%s\n' \
      'schema=1' \
      'mode=open' \
      "store=$scratch/agora-store" \
      "socket=$scratch/agora.sock" \
      "aegis_socket=$scratch/aegis.sock" \
      'participant=alice' \
      'subject=02020202020202020202020202020202' \
      'resource=07070707070707070707070707070707' \
      "authority_epoch=$epoch" \
      "grant_now=$grant_now" \
      "grant_expires_at=$grant_expires_at" \
      'grant_profile=scheduled' \
      'host_scope=08080808080808080808080808080808' \
      "server_incarnation=$incarnation" \
      >"$config"
  fi

  printf '%s\n' \
    '{' \
    '  "version": 1,' \
    '  "name": "Hosted Agora qualification",' \
    '  "surfaces": ["agora", "aegis"],' \
    '  "agora": {' \
    '    "deployment": "hosted",' \
    '    "transport": "composition",' \
    "    \"socket\": \"$scratch/agora.sock\"," \
    '    "participant": "alice",' \
    '    "requires": ["rooms.read.v2"],' \
    '    "host": {' \
    "      \"executable\": \"$agora\"," \
    "      \"config\": \"$config\"" \
    '    }' \
    '  },' \
    '  "aegis": {' \
    '    "deployment": "hosted",' \
    "    \"socket\": \"$scratch/aegis.sock\"," \
    '    "requires": ["authority.observe.v1"],' \
    '    "host": {' \
    "      \"executable\": \"$aegis\"," \
    "      \"directory\": \"$scratch/authority\"," \
    "      \"mode\": \"$mode\"," \
    '      "authority": "01010101010101010101010101010101",' \
    "      \"effective_uid\": $uid," \
    "      \"effective_gid\": $gid," \
    '      "principal": "02020202020202020202020202020202",' \
    '      "session_namespace": "0303030303030303",' \
    '      "role": "administrator"' \
    '    }' \
    '  }' \
    '}' >"$profile"
  if [ "$operation" = prepare ]; then
    printf '%s\n' "$profile"
    return
  fi
  report="$scratch/$mode-$epoch-report.json"
  if ! "$workbench" headless --workspace-profile "$profile" >"$report"; then
    cat "$report" >&2
    return 1
  fi
  cat "$report"
}

created=$(run_profile create 0 09090909090909090909090909090909)
printf '%s' "$created" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peers = {peer["peer"]: peer for peer in report["peers"]}
assert peers["agora"]["status"] == "current"
assert peers["agora"]["rooms"] == 1
assert peers["aegis"]["status"] == "current"
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [
        {"peer": "agora", "instance": None, "disposition": "clean"},
        {"peer": "aegis", "instance": None, "disposition": "clean"},
    ],
}
'
[ ! -e "$scratch/agora.sock" ]
[ ! -e "$scratch/aegis.sock" ]

reopened=$(run_profile open 1 0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a)
printf '%s' "$reopened" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peers = {peer["peer"]: peer for peer in report["peers"]}
assert peers["agora"]["status"] == "current"
assert peers["agora"]["rooms"] == 1
assert peers["aegis"]["epoch"] == 1
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/agora.sock" ]
[ ! -e "$scratch/aegis.sock" ]

combined_profile=$(run_profile open 2 0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b prepare)
"$aegis" open "$scratch/authority" "$scratch/aegis.sock" \
  01010101010101010101010101010101 "$uid" "$gid" \
  02020202020202020202020202020202 0303030303030303 administrator \
  >"$scratch/aegis-interactive.out" 2>"$scratch/aegis-interactive.err" &
aegis_pid=$!
for _ in $(seq 1 400); do
  [ -S "$scratch/aegis.sock" ] && break
  if ! kill -0 "$aegis_pid" 2>/dev/null; then
    cat "$scratch/aegis-interactive.err" >&2
    echo 'Aegis stopped before interactive Agora readiness' >&2
    exit 1
  fi
  sleep 0.01
done
[ -S "$scratch/aegis.sock" ]
interactive_profile="$scratch/interactive.json"
printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted Agora interactive qualification",' \
  '  "surfaces": ["agora"],' \
  '  "agora": {' \
  '    "deployment": "hosted",' \
  '    "transport": "composition",' \
  "    \"socket\": \"$scratch/agora.sock\"," \
  '    "participant": "alice",' \
  '    "requires": ["rooms.read.v2"],' \
  '    "host": {' \
  "      \"executable\": \"$agora\"," \
  "      \"config\": \"$scratch/agora-open.conf\"" \
  '    }' \
  '  }' \
  '}' >"$interactive_profile"
python3 - "$scratch" "$workbench" "$interactive_profile" <<'PYTHON'
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
        capture_output=True,
        text=True,
        check=check,
        timeout=5,
    )

try:
    tmux("new-session", "-d", "-x", "120", "-y", "24", "-s", "agora", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "agora").stdout
        if "Agora" in screen and "Scheduled Room" in screen:
            break
        time.sleep(0.025)
    else:
        history = tmux("capture-pane", "-p", "-S", "-100", "-t", "agora").stdout
        panes = tmux(
            "list-panes",
            "-t",
            "agora",
            "-F",
            "#{pane_dead} #{pane_current_command} #{pane_dead_status}",
        ).stdout
        raise AssertionError(f"Hosted Agora pane not ready ({panes.strip()}):\n{history}")

    tmux("send-keys", "-t", "agora", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "agora", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Workbench did not drain hosted Agora")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/agora.sock" ]
kill -TERM "$aegis_pid"
wait "$aegis_pid"
aegis_pid=''
[ ! -e "$scratch/aegis.sock" ]

after_interactive=$(run_profile open 3 0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c)
printf '%s' "$after_interactive" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peers = {peer["peer"]: peer for peer in report["peers"]}
assert peers["agora"]["rooms"] == 1
assert peers["aegis"]["epoch"] == 3
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/agora.sock" ]
[ ! -e "$scratch/aegis.sock" ]

echo 'PASS: Workbench-hosted Aegis -> exact Agora standalone process -> public client -> headless/interactive clean durable reopen'
