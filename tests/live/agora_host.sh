#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
scratch=$(mktemp -d /tmp/wb-agora.XXXXXX)
tmux_socket="$scratch/tmux.sock"
target="$scratch/target"
session=workbench-agora
server_pid=

cleanup() {
  if [ -n "$server_pid" ] && kill -0 "$server_pid" 2>/dev/null; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  tmux -S "$tmux_socket" kill-server >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo test --manifest-path "$repo/qualification/agora-host/Cargo.toml" \
  --all-targets --locked --offline --target-dir "$target/agora-owner"

cargo build --manifest-path "$repo/Cargo.toml" --package workbench-tui-cli --bin workbench-tui \
  --example agora_host --example agora_fixture_server \
  --locked --offline --target-dir "$target"

tmux -S "$tmux_socket" -f /dev/null new-session -d -x 120 -y 24 \
  -s "$session" "$target/debug/examples/agora_host"

expect() {
  marker=$1
  attempt=0
  while [ "$attempt" -lt 200 ]; do
    screen=$(tmux -S "$tmux_socket" capture-pane -p -t "$session")
    if printf '%s' "$screen" | grep -Fq "$marker"; then
      return
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
  printf '%s\n' "$screen" >&2
  echo "FAIL: Agora host omitted $marker" >&2
  exit 1
}

expect 'AGORA · rooms · revision 7'
expect 'Atlas Room'
tmux -S "$tmux_socket" send-keys -t "$session" Enter
expect 'Workbench Agora peer is live'
tmux -S "$tmux_socket" send-keys -t "$session" q
expect 'Use Ctrl-Q to exit the complete workbench'
tmux -S "$tmux_socket" send-keys -t "$session" C-q

attempt=0
while tmux -S "$tmux_socket" has-session -t "$session" 2>/dev/null; do
  if [ "$attempt" -eq 200 ]; then
    echo 'FAIL: Agora Workbench host did not exit' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

echo 'PASS: host-supplied Agora peer -> Workbench surface'

session=workbench-agora-profile
agora_socket="$scratch/agora.sock"
profile="$scratch/workspace.json"
server_log="$scratch/agora-server.log"
printf '%s\n' \
  '{"version":1,"name":"Agora profile gate","surfaces":["agora"],"agora":{"socket":"agora.sock","participant":"operator"},"chronos":null,"vellum":null}' \
  >"$profile"
"$target/debug/examples/agora_fixture_server" "$agora_socket" >"$server_log" 2>&1 &
server_pid=$!

attempt=0
while [ ! -S "$agora_socket" ]; do
  if ! kill -0 "$server_pid" 2>/dev/null || [ "$attempt" -eq 200 ]; then
    sed -n '1,120p' "$server_log" >&2
    echo 'FAIL: Agora fixture server did not become ready' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

tmux -S "$tmux_socket" -f /dev/null new-session -d -x 120 -y 24 \
  -s "$session" "$target/debug/workbench-tui --workspace-profile $profile"
expect 'AGORA · rooms · revision 11'
expect 'Profile Room'
tmux -S "$tmux_socket" send-keys -t "$session" Enter
expect 'Profile Agora transport is live'
tmux -S "$tmux_socket" send-keys -t "$session" C-q

attempt=0
while tmux -S "$tmux_socket" has-session -t "$session" 2>/dev/null; do
  if [ "$attempt" -eq 200 ]; then
    echo 'FAIL: profile-driven Agora Workbench did not exit' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done
wait "$server_pid"
server_pid=

echo 'PASS: profile-driven Agora Unix client -> Workbench surface'

rm -f "$agora_socket"
"$target/debug/examples/agora_fixture_server" "$agora_socket" >"$server_log" 2>&1 &
server_pid=$!
attempt=0
while [ ! -S "$agora_socket" ]; do
  if ! kill -0 "$server_pid" 2>/dev/null || [ "$attempt" -eq 200 ]; then
    sed -n '1,120p' "$server_log" >&2
    echo 'FAIL: Agora headless fixture did not become ready' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done
headless=$("$target/debug/workbench-tui" headless --workspace-profile "$profile")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"] and report["mode"] == "headless"
assert report["peers"] == [{
    "capability": "rooms.read.v2",
    "peer": "agora",
    "revision": 11,
    "rooms": 1,
    "status": "current",
    "truncated": False,
}]
'
wait "$server_pid"
server_pid=

echo 'PASS: profile-driven Agora Unix client -> Workbench headless observation'
