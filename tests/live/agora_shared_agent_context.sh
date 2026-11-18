#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pasac.XXXXXX")
agora_pid=
link_pid=

cleanup() {
  for pid in "$agora_pid" "$link_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir -m 700 "$scratch/chronos"

cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  --package daimon-tui-cli --example agent_set_fixture --locked --offline
cargo build --manifest-path "$workspace/agora-tui/Cargo.toml" \
  --package agora-tui-cli --example real_daemon_fixture --locked --offline
cargo build --manifest-path "$workspace/agora-chronos-adapter/Cargo.toml" \
  --bin task-conversation-service --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

agora_fixture="$workspace/agora-tui/target/debug/examples/real_daemon_fixture"
link_service="$workspace/agora-chronos-adapter/target/debug/task-conversation-service"
workbench="$root/target/debug/pantheon"
catalog_owner="$root/tests/support/provider_catalog_shared_context_owner.sh"

wait_socket() {
  path=$1
  pid=$2
  log=$3
  attempt=0
  while [ ! -S "$path" ]; do
    if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 400 ]; then
      sed -n '1,160p' "$log" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

"$agora_fixture" "$scratch/agora.sock" \
  >"$scratch/agora.out" 2>"$scratch/agora.err" &
agora_pid=$!
wait_socket "$scratch/agora.sock" "$agora_pid" "$scratch/agora.err"

"$link_service" "$scratch/link-state" "$scratch/task-conversation.sock" \
  >"$scratch/link.out" 2>"$scratch/link.err" &
link_pid=$!
wait_socket "$scratch/task-conversation.sock" "$link_pid" "$scratch/link.err"

cat >"$scratch/provider-catalog.json" <<'EOF'
{
  "version": 1,
  "owner": "daimon-local-agent",
  "document_digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  "providers": [{
    "provider_id": "22222222222222222222222222222222",
    "harness_profile_id": "11111111111111111111111111111111",
    "profile_document_digest": "sha256:3333333333333333333333333333333333333333333333333333333333333333",
    "policy_digest": "sha256:4444444444444444444444444444444444444444444444444444444444444444",
    "transport": "acp",
    "isolation": "direct",
    "executable_available": true,
    "argument_count": 0,
    "inherited_environment_name_count": 0,
    "authentication_method_configured": false
  }],
  "task_bindings": [{
    "external_task_key": "14141414141414141414141414141414",
    "chronos_task_id": "recurring-incident"
  }],
  "contexts": []
}
EOF

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Agora shared agent context qualification",
  "surfaces": ["chronos", "agora", "daimon"],
  "agora": {
    "socket": "$scratch/agora.sock",
    "participant": "operator",
    "requires": ["rooms.read.v2"]
  },
  "task_conversation": {
    "socket": "$scratch/task-conversation.sock",
    "principal": "10101010101010101010101010101010",
    "agent_routes": [
      {"addressee": "sre-agent", "session": "07070707070707070707070707070707"},
      {"addressee": "project-agent", "session": "08080808080808080808080808080808"}
    ],
    "requires": ["task-conversation.v1"]
  },
  "chronos": {
    "deployment": "hosted",
    "workstreams": [{"name": "Operations", "directory": "$scratch/chronos"}],
    "requires": ["tasks.observe.v1"],
    "moira": [],
    "daimon": [],
    "planning": null
  },
  "daimon": {
    "deployment": "hosted",
    "agent_socket": "$scratch/daimon.sock",
    "agent_sessions": [
      "07070707070707070707070707070707",
      "08080808080808080808080808080808",
      "09090909090909090909090909090909"
    ],
    "execution": {
      "principal": "10101010101010101010101010101010",
      "profile": "11111111111111111111111111111111",
      "task_attempt": {"task_key": "14141414141414141414141414141414", "task_incarnation": 1, "attempt_number": 1},
      "context": ["12121212121212121212121212121212"],
      "tools": [],
      "budget": {
        "max_elapsed_millis": 60000,
        "max_input_tokens": 4096,
        "max_output_tokens": 8192,
        "max_tool_calls": 32,
        "max_context_bytes": 1048576,
        "max_output_bytes": 1048576
      }
    },
    "host": {
      "executable": "$catalog_owner",
      "config": "$scratch/provider-catalog.json"
    },
    "requires": ["agents.session-set.inspect.v1", "agents.session-set.prompt.v1"]
  }
}
EOF

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([
    binary, "interactive", "--workspace-profile", profile,
    "--chronos-allow-writes",
]) + " 2>" + shlex.quote(str(Path(scratch) / "pantheon.err"))
session = "shared-agent-context"

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    captured = tmux("capture-pane", "-p", "-t", session, check=False)
    if captured.returncode != 0:
        error = Path(scratch, "pantheon.err").read_text(errors="replace")
        fixture = Path(scratch, "fixture.err").read_text(errors="replace")
        raise AssertionError(f"Pantheon exited early:\n{error}\nFixture:\n{fixture}")
    return captured.stdout

def expect(*markers, timeout=20):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    error = Path(scratch, "pantheon.err").read_text(errors="replace")
    fixture = Path(scratch, "fixture.err").read_text(errors="replace")
    raise AssertionError(f"Missing {markers!r}:\n{visible}\nPantheon:\n{error}\nFixture:\n{fixture}")

def keys(*values):
    tmux("send-keys", "-t", session, *values)

def literal(value):
    tmux("send-keys", "-t", session, "-l", value)

def palette(query, expected):
    keys("C-p", "C-u")
    literal(query)
    expect(expected)
    keys("Enter")

try:
    tmux("new-session", "-d", "-x", "180", "-y", "40", "-s", session, command)
    expect("CHRONOS", "Board", "revision 0", timeout=35)
    palette("go to daimon", "Go to Daimon")
    expect("session  1  Ready", "session  2  Ready", "session  3  Ready")
    keys("Right", "Right", "Space")
    expect("1 session(s) selected for bounded work")

    palette("create task with product owner agent lens", "Create task with product owner agent lens")
    expect("Task batch")
    literal("recurring-incident")
    keys("Enter")
    expect("Review 1 exact task(s)")
    keys("Enter")
    expect(
        "Compose Daimon prompt",
        "product-owner collaboration lens",
        "recurring-incident",
        timeout=20,
    )
    keys("Tab", "c", "Enter")
    expect("Daimon guarded session set", "1 exact session controls")
    keys("c", "Enter")
    expect(
        "Create Agora room",
        "group direct · operator, project-agent, sre-agent",
        timeout=20,
    )
    keys("c", "Enter")
    expect("Bind Chronos task to Agora room", "human gate", timeout=20)
    keys("c", "Enter")
    expect(
        "BOUND · Owner created relationship",
        "Agora room recurring-incident durably created",
        timeout=20,
    )
    keys("Escape")
    expect("Create Agora room", "Applied", timeout=20)
    keys("Escape")
    expect("Live Room", "Task · recurring-incident", timeout=20)
    keys("Down", "Enter")
    expect("Conversation · recurring-incident", timeout=20)

    palette("compose an agora message", "Compose an Agora message")
    expect("Compose Agora message")
    literal("Do we have the right ensemble? SRE reports a recurring issue; project management proposes deprioritization. Make the case. This deliberation does not authorize disposition.")
    keys("Tab")
    literal("sre-agent,project-agent")
    keys("Tab", "c", "Enter")
    expect("Agora accepted message #", timeout=20)
    keys("Escape")
    expect("Do we have the right ensemble?", "to: project-agent, sre-agent", timeout=20)

    palette("offer addressed agora message to agent cohort", "Offer addressed Agora message to agent cohort")
    expect(
        "Compose Daimon prompt",
        "2 selected session(s)",
        "Configured addressees: sre-agent, project-agent",
        "Do we have the right ensemble?",
        timeout=20,
    )
    keys("Tab", "c", "Enter")
    expect("Daimon guarded session set", "2 exact session controls")
    keys("c", "Enter")
    expect(
        "Addressed message accepted",
        "Routed turns applied",
        "ambient chatter inert",
        timeout=20,
    )
    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", session, check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after the shared-context journey")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

wait "$agora_pid"
agora_pid=
"$agora_fixture" "$scratch/agora.sock" \
  >"$scratch/agora-headless.out" 2>"$scratch/agora-headless.err" &
agora_pid=$!
wait_socket "$scratch/agora.sock" "$agora_pid" "$scratch/agora-headless.err"
headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
link = next(peer for peer in report["peers"] if peer["peer"] == "task-conversation")
assert link["status"] == "current" and link["links"] == 1
assert link["agent_routing"] == {
    "mode": "exact_participant_sessions",
    "routes": [
        {"addressee": "sre-agent", "session": "07070707070707070707070707070707"},
        {"addressee": "project-agent", "session": "08080808080808080808080808080808"},
    ],
}
assert "grants neither Agora membership" in link["routing_boundary"]
'
wait "$agora_pid"
agora_pid=

test "$(wc -l <"$scratch/receipts" | tr -d ' ')" = 3
grep 'session 1 prompted' "$scratch/receipts" >/dev/null
grep 'session 2 prompted' "$scratch/receipts" >/dev/null
grep 'session 3 prompted' "$scratch/receipts" >/dev/null

echo 'PASS: third agent handles work item -> human-gated group-direct Agora context -> exact SRE and project-management participant routes -> two routed Daimon turns -> ambient session excluded -> headless routing parity'
