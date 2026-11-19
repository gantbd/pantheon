#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pantheon-agora-observe.XXXXXX")
socket="$scratch/agora.sock"
daemon_pid=

cleanup() {
  if [ -n "$daemon_pid" ] && kill -0 "$daemon_pid" 2>/dev/null; then
    kill "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --quiet --manifest-path "$workspace/agora-tui/Cargo.toml" \
  --package agora-tui-cli --example real_daemon_fixture --locked --offline
cargo build --quiet --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

daemon="$workspace/agora-tui/target/debug/examples/real_daemon_fixture"
pantheon="$root/target/debug/pantheon"

start_daemon() {
  rm -f "$socket"
  "$daemon" "$socket" >"$scratch/daemon.out" 2>"$scratch/daemon.err" &
  daemon_pid=$!
  attempt=0
  while [ ! -S "$socket" ]; do
    attempt=$((attempt + 1))
    if ! kill -0 "$daemon_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
      sed -n '1,160p' "$scratch/daemon.err" >&2
      exit 1
    fi
    sleep 0.025
  done
}

finish_daemon() {
  wait "$daemon_pid"
  daemon_pid=
}

start_daemon
"$pantheon" agora-room-observe "$socket" operator conversation live-room \
  >"$scratch/room.json"
finish_daemon

start_daemon
"$pantheon" agora-message-observe "$socket" operator conversation live-room - 16 \
  >"$scratch/messages.json"
finish_daemon

start_daemon
if "$pantheon" agora-room-observe "$socket" operator conversation missing-room \
  >"$scratch/missing.json" 2>"$scratch/missing.err"; then
  echo "missing Agora room was accepted" >&2
  exit 1
fi
finish_daemon

if "$pantheon" agora-message-observe "$scratch/missing.sock" operator conversation \
  live-room - 65 >"$scratch/limit.json" 2>"$scratch/limit.err"; then
  echo "Agora message limit above 64 was accepted" >&2
  exit 1
fi

python3 - "$scratch" <<'PY'
from pathlib import Path
import json, sys

scratch = Path(sys.argv[1])
room = json.loads((scratch / "room.json").read_text())
assert room["capability"] == "rooms.read.v2"
assert room["complete"] is True
assert room["evidence"]["room"] == {
    "id": "live-room",
    "name": "Live Room",
    "admission": "invite_only",
    "observer_role": "owner",
    "member_count": 1,
    "retained_message_count": 1,
    "next_message_sequence": 1,
}
assert "rooms" not in room["evidence"]

messages = json.loads((scratch / "messages.json").read_text())
assert messages["capability"] == "messages.read.v2"
assert messages["complete"] is True
assert messages["evidence"]["room"] == "live-room"
assert messages["evidence"]["count"] == 1
message = messages["evidence"]["messages"][0]
assert message["author"] == "operator"
assert message["body"] == "Standalone CLI reached the real Agora daemon"
assert message["addressed_to"] == []
assert "intentionally disclosed" in messages["boundary"]

missing = json.loads((scratch / "missing.json").read_text())
assert missing["capability"] == "rooms.read.v2"
assert missing["complete"] is False
assert missing["status"] == "not_found"
assert "live-room" not in (scratch / "missing.json").read_text()

limit = json.loads((scratch / "limit.json").read_text())
assert limit["complete"] is False
assert limit["status"] == "configuration_refused"
assert "must be 1..=64" in limit["detail"]
PY

echo "PASS: exact Agora room and bounded message observations preserve component admission"
