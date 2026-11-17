#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-operational.XXXXXX")
moira_pid=
daimon_pid=
daimon_agent_pid=
hermes_pid=
aegis_pid=
dendrite_pid=

cleanup() {
  for pid in "$moira_pid" "$daimon_pid" "$daimon_agent_pid" "$hermes_pid" "$aegis_pid" "$dendrite_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

build_fixture() {
  peer=$1
  package=$2
  cargo build --manifest-path "$workspace/$peer-tui/Cargo.toml" \
    --package "$package" --example real_daemon_fixture \
    --locked --offline
}

build_fixture moira moira-tui-cli
build_fixture daimon daimon-tui-cli
cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  --package daimon-tui-cli --example agent_daemon_fixture \
  --locked --offline
build_fixture hermes hermes-tui-cli
build_fixture aegis aegis-tui-cli
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon \
  --locked --offline

"$workspace/moira-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/moira.db" "$scratch/moira.sock" >"$scratch/moira.log" 2>&1 &
moira_pid=$!
"$workspace/daimon-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/daimon-store" "$scratch/daimon.sock" >"$scratch/daimon.log" 2>&1 &
daimon_pid=$!
"$workspace/hermes-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/hermes-ledger" "$scratch/hermes.sock" >"$scratch/hermes.log" 2>&1 &
hermes_pid=$!
"$workspace/aegis-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/aegis-authority" "$scratch/aegis.sock" >"$scratch/aegis.log" 2>&1 &
aegis_pid=$!
(cd "$workspace/dendrite" && \
  DENDRITE_GROUNDING_PORT_FILE="$scratch/grounding.port" MIX_ENV=test \
  mix run scripts/grounding_fixture.exs) >"$scratch/dendrite.log" 2>&1 &
dendrite_pid=$!

for peer in moira daimon hermes aegis; do
  attempt=0
  while [ ! -S "$scratch/$peer.sock" ]; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 100 ]; then
      sed -n '1,120p' "$scratch/$peer.log" >&2
      echo "FAIL: $peer fixture did not become ready" >&2
      exit 1
    fi
    sleep 0.05
  done
done

attempt=0
while [ ! -s "$scratch/grounding.port" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$dendrite_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/dendrite.log" >&2
    echo "FAIL: Grounding fixture did not become ready" >&2
    exit 1
  fi
  sleep 0.05
done
grounding_endpoint="127.0.0.1:$(cat "$scratch/grounding.port")"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Operational peer coexistence qualification",' \
  '  "surfaces": ["overview", "moira", "daimon", "hermes", "aegis", "grounding"],' \
  "  \"moira\": {\"socket\": \"$scratch/moira.sock\", \"requires\": [\"sagas.inspect.v1\"]}," \
  "  \"daimon\": {\"socket\": \"$scratch/daimon.sock\", \"requires\": [\"agents.presence.v1\"]}," \
  "  \"hermes\": {\"socket\": \"$scratch/hermes.sock\", \"stream\": \"05050505050505050505050505050505\", \"consumer\": \"06060606060606060606060606060606\", \"requires\": [\"ledger.observe.v1\", \"ledger.cursor.observe.v1\"]}," \
  "  \"aegis\": {\"socket\": \"$scratch/aegis.sock\", \"requires\": [\"authority.observe.v1\"]}," \
  "  \"grounding\": {\"endpoint\": \"$grounding_endpoint\", \"token\": null, \"requires\": [\"grounding.observe.v1\"]}" \
  '}' >"$scratch/workspace.json"

report=$("$root/target/debug/pantheon" \
  headless --workspace-profile "$scratch/workspace.json")

wait "$moira_pid"; moira_pid=
wait "$daimon_pid"; daimon_pid=
wait "$hermes_pid"; hermes_pid=
wait "$aegis_pid"; aegis_pid=
kill "$dendrite_pid"
wait "$dendrite_pid" 2>/dev/null || true
dendrite_pid=

printf '%s' "$report" | python3 -c '
import json
import sys

report = json.load(sys.stdin)
assert report["version"] == 1
assert report["mode"] == "headless"
assert report["complete"]
expected = [
    ("moira", "sagas.inspect.v1"),
    ("daimon", "agents.presence.v1"),
    ("hermes", "ledger.observe.v1"),
    ("aegis", "authority.observe.v1"),
    ("grounding", "grounding.observe.v1"),
]
actual = [(peer["peer"], peer["capability"]) for peer in report["peers"]]
assert actual == expected, actual
assert all(peer["status"] == "current" for peer in report["peers"])
moira = next(peer for peer in report["peers"] if peer["peer"] == "moira")
assert moira["census"]["total"] == 1
assert moira["census"]["fenced"] == 1
hermes = next(peer for peer in report["peers"] if peer["peer"] == "hermes")
assert hermes["configured_stream"]["status"] == "current"
assert hermes["configured_stream"]["retained_events"] == 1
assert hermes["configured_cursor"]["status"] == "current"
assert hermes["configured_cursor"]["next_sequence"] == 1
assert hermes["configured_cursor"]["backlog"] == 0
'

"$workspace/moira-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/moira-interactive.db" "$scratch/moira-interactive.sock" >"$scratch/moira-interactive.log" 2>&1 &
moira_pid=$!
"$workspace/daimon-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/daimon-interactive-store" "$scratch/daimon-interactive.sock" >"$scratch/daimon-interactive.log" 2>&1 &
daimon_pid=$!
"$workspace/daimon-tui/target/debug/examples/agent_daemon_fixture" \
  "$scratch/agent.sock" >"$scratch/daimon-agent-interactive.log" 2>&1 &
daimon_agent_pid=$!
"$workspace/hermes-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/hermes-interactive-ledger" "$scratch/hermes-interactive.sock" >"$scratch/hermes-interactive.log" 2>&1 &
hermes_pid=$!
"$workspace/aegis-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/aegis-interactive-authority" "$scratch/aegis-interactive.sock" >"$scratch/aegis-interactive.log" 2>&1 &
aegis_pid=$!
(cd "$workspace/dendrite" && \
  DENDRITE_GROUNDING_PORT_FILE="$scratch/grounding-interactive.port" MIX_ENV=test \
  mix run scripts/grounding_fixture.exs) >"$scratch/dendrite-interactive.log" 2>&1 &
dendrite_pid=$!

for peer in moira daimon hermes aegis; do
  attempt=0
  while [ ! -S "$scratch/$peer-interactive.sock" ]; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 100 ]; then
      sed -n '1,120p' "$scratch/$peer-interactive.log" >&2
      echo "FAIL: $peer interactive fixture did not become ready" >&2
      exit 1
    fi
    sleep 0.05
  done
done

attempt=0
while [ ! -S "$scratch/agent.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$daimon_agent_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,120p' "$scratch/daimon-agent-interactive.log" >&2
    echo "FAIL: Daimon agent interactive fixture did not become ready" >&2
    exit 1
  fi
  sleep 0.05
done

attempt=0
while [ ! -s "$scratch/grounding-interactive.port" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$dendrite_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/dendrite-interactive.log" >&2
    echo "FAIL: interactive Grounding fixture did not become ready" >&2
    exit 1
  fi
  sleep 0.05
done
grounding_endpoint="127.0.0.1:$(cat "$scratch/grounding-interactive.port")"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Operational peer Overview qualification",' \
  '  "surfaces": ["overview", "moira", "daimon", "hermes", "aegis", "grounding"],' \
  "  \"moira\": {\"socket\": \"$scratch/moira-interactive.sock\", \"requires\": [\"sagas.inspect.v1\"]}," \
  "  \"daimon\": {\"socket\": \"$scratch/daimon-interactive.sock\", \"agent_socket\": \"$scratch/agent.sock\", \"agent_session\": \"07070707070707070707070707070707\", \"requires\": [\"agents.presence.v1\", \"agents.session.inspect.v1\"]}," \
  "  \"hermes\": {\"socket\": \"$scratch/hermes-interactive.sock\", \"stream\": \"05050505050505050505050505050505\", \"consumer\": \"06060606060606060606060606060606\", \"requires\": [\"ledger.observe.v1\", \"ledger.cursor.observe.v1\"]}," \
  "  \"aegis\": {\"socket\": \"$scratch/aegis-interactive.sock\", \"requires\": [\"authority.observe.v1\"]}," \
  "  \"grounding\": {\"endpoint\": \"$grounding_endpoint\", \"token\": null, \"requires\": [\"grounding.observe.v1\"]}" \
  '}' >"$scratch/interactive-workspace.json"

python3 - "$scratch" "$root/target/debug/pantheon" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "interactive-workspace.json")
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
    tmux("new-session", "-d", "-x", "180", "-y", "32", "-s", "overview", command)
    markers = [
        "moira                 sagas",
        "daimon                presence",
        "daimon                sessions/events",
        "hermes                ledger/cursors",
        "aegis                 authority",
        "grounding             provenance",
    ]
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "overview").stdout
        if all(marker in screen for marker in markers):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Pantheon Overview did not settle all evidence channels:\n{screen}")
    tmux("send-keys", "-t", "overview", "]", "]", "f", "f", "f", "f")
    detail_markers = [
        "Daimon session evidence",
        "events: 1 visible / 2 observed · filter usage",
        "set 1 configured · 1 current · 0 unavailable",
        "observed windows 1/2 events visible · usage input=21 output=34",
    ]
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "overview").stdout
        if all(marker in screen for marker in detail_markers):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Pantheon did not expose filtered Daimon workset evidence:\n{screen}")
    tmux("send-keys", "-t", "overview", "C-p")
    tmux("send-keys", "-t", "overview", "-l", "inspect tui instrumentation")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "overview").stdout
        if "Inspect TUI instrumentation" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Pantheon did not discover instrumentation:\n{screen}")
    tmux("send-keys", "-t", "overview", "Enter")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "overview").stdout
        if all(marker in screen for marker in [
            "TUI instrumentation",
            "0 in flight · 0 budget violations",
            "workbench.render",
        ]):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Pantheon did not retain performant filtered rendering:\n{screen}")
    tmux("send-keys", "-t", "overview", "Escape")
    tmux("send-keys", "-t", "overview", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "overview", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

wait "$moira_pid"; moira_pid=
wait "$daimon_pid"; daimon_pid=
wait "$daimon_agent_pid"; daimon_agent_pid=
wait "$hermes_pid"; hermes_pid=
wait "$aegis_pid"; aegis_pid=
kill "$dendrite_pid"
wait "$dendrite_pid" 2>/dev/null || true
dendrite_pid=

echo 'PASS: Moira + Daimon + Hermes + Aegis + Grounding real services -> headless composition and evidence-backed Ratatui Overview'
