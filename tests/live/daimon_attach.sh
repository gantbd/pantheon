#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
workspace=$(CDPATH= cd -- "$root/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon.XXXXXX")
presence_pid=
agent_pid=

cleanup() {
  for pid in "$presence_pid" "$agent_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

cargo build --quiet --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  -p daimon-tui-cli --example real_daemon_fixture --example agent_daemon_fixture \
  --locked --offline
cargo build --quiet --manifest-path "$root/Cargo.toml" \
  -p workbench-tui-cli --bin workbench-tui --locked --offline
presence_fixture="$workspace/daimon-tui/target/debug/examples/real_daemon_fixture"
agent_fixture="$workspace/daimon-tui/target/debug/examples/agent_daemon_fixture"
workbench="$root/target/debug/workbench-tui"
agent_session=07070707070707070707070707070707

start_fixtures() {
  name=$1
  "$presence_fixture" "$tmp/$name-store" "$tmp/$name-presence.sock" \
    >"$tmp/$name-presence.out" 2>"$tmp/$name-presence.err" &
  presence_pid=$!
  "$agent_fixture" "$tmp/$name-agent.sock" \
    >"$tmp/$name-agent.out" 2>"$tmp/$name-agent.err" &
  agent_pid=$!
  attempt=0
  while [ ! -S "$tmp/$name-presence.sock" ] || [ ! -S "$tmp/$name-agent.sock" ]; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 100 ]; then
      cat "$tmp/$name-presence.err" "$tmp/$name-agent.err" >&2
      exit 1
    fi
    sleep 0.05
  done
}

write_profile() {
  name=$1
  printf '%s\n' \
    '{' \
    '  "version": 1,' \
    '  "name": "Daimon attach qualification",' \
    '  "surfaces": ["daimon"],' \
    "  \"daimon\": {\"socket\": \"$tmp/$name-presence.sock\", \"agent_socket\": \"$tmp/$name-agent.sock\", \"agent_session\": \"$agent_session\", \"requires\": [\"agents.presence.v1\", \"agents.session.inspect.v1\"]}," \
    '  "agora": null,' \
    '  "moira": null,' \
    '  "chronos": null,' \
    '  "vellum": null' \
    '}' >"$tmp/$name-profile.json"
}

start_fixtures headless
write_profile headless
output=$("$workbench" headless --workspace-profile "$tmp/headless-profile.json")
wait "$presence_pid"; presence_pid=
wait "$agent_pid"; agent_pid=

printf '%s\n' "$output" | grep '"complete": true' >/dev/null
printf '%s\n' "$output" | grep '"peer": "daimon"' >/dev/null
printf '%s\n' "$output" | grep '"capability": "agents.presence.v1"' >/dev/null
printf '%s\n' "$output" | grep '"agents.session.inspect.v1"' >/dev/null
printf '%s\n' "$output" | grep '"phase": "ReattachRequired"' >/dev/null
printf '%s\n' "$output" | grep '"kind": "plan"' >/dev/null

start_fixtures interactive
write_profile interactive
python3 - "$tmp" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", str(Path(scratch) / "interactive-profile.json")])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

try:
    tmux("new-session", "-d", "-x", "140", "-y", "18", "-s", "workbench", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "workbench").stdout
        if all(marker in screen for marker in [
            "Daimon agent", "ReattachRequired/12", "plan inspect before resuming",
        ]):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Workbench did not render Daimon agent evidence:\n{screen}")
    tmux("send-keys", "-t", "workbench", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "workbench", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Workbench did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
wait "$presence_pid"; presence_pid=
wait "$agent_pid"; agent_pid=

echo 'PASS: real Daimon presence and exact configured agent-session evidence -> Workbench headless and interactive attach'
