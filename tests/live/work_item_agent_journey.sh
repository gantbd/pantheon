#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
# Unix-domain endpoints have a small path ceiling on macOS; keep the complete
# Chronos/Daimon fixture topology under the short private temp root.
scratch=$(mktemp -d "/private/tmp/pwiaj.XXXXXX")
fixture_pid=
agora_pid=
link_pid=

cleanup() {
  for pid in "$fixture_pid" "$agora_pid" "$link_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  if [ "${PANTHEON_KEEP_J1_SCRATCH:-0}" = 1 ]; then
    echo "J1 scratch retained for diagnosis: $scratch" >&2
  else
    rm -rf "$scratch"
  fi
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/chronos"
chmod 700 "$scratch/chronos"

qualified_daimon_fixture=${PANTHEON_DAIMON_JOURNEY_FIXTURE:-}
if [ -n "$qualified_daimon_fixture" ]; then
  expected_digest=${PANTHEON_DAIMON_JOURNEY_FIXTURE_SHA256:-}
  provenance=${PANTHEON_DAIMON_JOURNEY_FIXTURE_PROVENANCE:-}
  if [ -z "$expected_digest" ] || [ -z "$provenance" ]; then
    echo 'FAIL: qualified Daimon fixture requires exact SHA-256 and provenance' >&2
    exit 1
  fi
  if [ ! -f "$qualified_daimon_fixture" ] || [ ! -x "$qualified_daimon_fixture" ]; then
    echo "FAIL: qualified Daimon fixture is not an executable file: $qualified_daimon_fixture" >&2
    exit 1
  fi
  actual_digest=$(shasum -a 256 "$qualified_daimon_fixture" | awk '{print $1}')
  if [ "$actual_digest" != "$expected_digest" ]; then
    echo "FAIL: qualified Daimon fixture digest mismatch: expected $expected_digest, observed $actual_digest" >&2
    exit 1
  fi
  fixture=$qualified_daimon_fixture
  echo "QUALIFIED ARTIFACT: Daimon fixture sha256:$actual_digest · $provenance"
else
  if ! cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
    --package daimon-tui-cli --example agent_open_journey_fixture --locked --offline; then
    echo 'FAIL: Daimon fixture owner locked graph is not buildable.' >&2
    echo 'Pantheon did not update or bypass the peer lockfile.' >&2
    echo 'Next: reconcile daimon-tui/Cargo.lock in its owner repository, then rerun make qualify-j1.' >&2
    exit 1
  fi
  fixture="$workspace/daimon-tui/target/debug/examples/agent_open_journey_fixture"
fi
cargo build --manifest-path "$workspace/agora-tui/Cargo.toml" \
  --package agora-tui-cli --example real_daemon_fixture --locked --offline
cargo build --manifest-path "$workspace/agora-chronos-adapter/Cargo.toml" \
  --bin task-conversation-service --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

workbench="$root/target/debug/pantheon"
catalog_owner="$root/tests/support/provider_catalog_open_owner.sh"
agora_fixture="$workspace/agora-tui/target/debug/examples/real_daemon_fixture"
link_service="$workspace/agora-chronos-adapter/target/debug/task-conversation-service"

wait_socket() {
  path=$1
  pid=$2
  log=$3
  attempt=0
  while [ ! -S "$path" ]; do
    if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 400 ]; then
      sed -n '1,160p' "$log" >&2
      echo "FAIL: service did not publish $path" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

start_agora() {
  "$agora_fixture" "$scratch/agora.sock" \
    >"$scratch/agora.out" 2>"$scratch/agora.err" &
  agora_pid=$!
  wait_socket "$scratch/agora.sock" "$agora_pid" "$scratch/agora.err"
}

start_link() {
  "$link_service" "$scratch/link-state" "$scratch/task-conversation.sock" \
    >"$scratch/link.out" 2>"$scratch/link.err" &
  link_pid=$!
  wait_socket "$scratch/task-conversation.sock" "$link_pid" "$scratch/link.err"
}

drain_link() {
  python3 - "$scratch/task-conversation.sock" <<'PY'
import json, socket, sys
with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as stream:
    stream.settimeout(3)
    stream.connect(sys.argv[1])
    stream.sendall(json.dumps({"operation": "drain"}).encode() + b"\n")
    reply = json.loads(stream.makefile("rb").readline(65537))
assert reply["complete"] and reply["status"] == "draining"
PY
  wait "$link_pid"
  link_pid=
}

inspect_relationship() {
  report="$scratch/task-conversation-inspection.json"
  if ! "$workbench" task-conversation-inspect \
    "$scratch/task-conversation.sock" conversation local-agent-task - >"$report"; then
    echo 'FAIL: exact task-conversation owner inspection was unavailable' >&2
    sed -n '1,160p' "$scratch/link.err" >&2
    exit 1
  fi
  python3 -c '
import json, sys
report = json.load(open(sys.argv[1]))
assert report["version"] == 1
assert report["mode"] == "task_conversation_exact_observation"
assert report["owner"] == "task-conversation.v1"
assert report["status"] == "current" and report["complete"]
assert report["owner_revision"] == 1
assert report["query"] == {
    "kind": "conversation",
    "reference": {"room": "local-agent-task", "thread_root": None},
}
assert report["link_count"] == 1 and len(report["links"]) == 1
link = report["links"][0]
assert len(link["link_id"]) == 32
assert link["task"]["task"] == "local-agent-task"
assert link["conversation"] == {"room": "local-agent-task", "thread_root": None}
assert len(link["operation_id"]) == 32 and len(link["principal_id"]) == 32
assert link["chronos_revision"] and link["agora_revision"] > 0
assert "not task progress" in report["boundary"]
' "$report"
}

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
    "chronos_task_id": "local-agent-task"
  }],
  "contexts": []
}
EOF

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Work item agent journey",
  "surfaces": ["chronos", "agora", "daimon"],
  "agora": {
    "socket": "$scratch/agora.sock",
    "participant": "operator",
    "requires": ["rooms.read.v2"]
  },
  "task_conversation": {
    "socket": "$scratch/task-conversation.sock",
    "principal": "10101010101010101010101010101010",
    "agent_addressee": "local-agent",
    "requires": ["task-conversation.v1"]
  },
  "chronos": {
    "deployment": "hosted",
    "workstreams": [{"name": "Local work", "directory": "$scratch/chronos"}],
    "requires": ["tasks.observe.v1"],
    "moira": [],
    "daimon": [],
    "planning": null
  },
  "daimon": {
    "deployment": "hosted",
    "agent_socket": "$scratch/daimon.sock",
    "agent_open": {
      "workspace": "15151515151515151515151515151515",
      "budget": {
        "max_elapsed_millis": 60000,
        "max_turns": 8,
        "max_events": 128,
        "max_event_bytes": 65536,
        "max_prompt_bytes": 16384,
        "max_permission_requests": 4
      }
    },
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
    "requires": ["agents.session.open.v1"]
  }
}
EOF

start_link
start_agora
headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
daimon = next(peer for peer in report["peers"] if peer["peer"] == "daimon")
execution = daimon["configured_execution_profile"]
assert execution["status"] == "selected"
assert execution["task_attempt"] == {
    "task_key": "14141414141414141414141414141414",
    "task_incarnation": 1,
    "attempt_number": 1,
}
assert execution["harness_profile"] == "11111111111111111111111111111111"
assert execution["context_count"] == 1 and execution["tool_count"] == 0
assert execution["budget"]["max_tool_calls"] == 32
for field in ("principal_type", "role_work_mode", "delegation", "accountable_owner", "authority_reference", "policy_epoch", "expiry"):
    assert execution[field] == "not_published"
assert "not role activation" in execution["boundary"]
assert report["not_exercised_profile_requirements"] == [
    {
        "peer": "daimon",
        "capability": "agents.session.open.v1",
        "disposition": "not_exercised",
        "reason": "aggregate_observation_only",
    },
]
assert "not a control receipt" in report["requirement_boundary"]
'

wait "$agora_pid"
agora_pid=
start_agora

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
session = "work-item-agent"

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    captured = tmux("capture-pane", "-p", "-t", session, check=False)
    if captured.returncode != 0:
        error = Path(scratch, "pantheon.err").read_text(errors="replace")
        raise AssertionError(f"Pantheon exited before the expected screen:\n{error}")
    return captured.stdout

def expect(*markers, timeout=15):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    error = Path(scratch, "pantheon.err").read_text(errors="replace")
    fixture_error = Path(scratch, "fixture.err").read_text(errors="replace") if Path(scratch, "fixture.err").exists() else ""
    raise AssertionError(f"Missing {markers!r} in:\n{visible}\nPantheon stderr:\n{error}\nFixture stderr:\n{fixture_error}")

def keys(*values):
    tmux("send-keys", "-t", session, *values)

def literal(value):
    tmux("send-keys", "-t", session, "-l", value)

try:
    tmux("new-session", "-d", "-x", "170", "-y", "34", "-s", session, command)
    expect("CHRONOS", "Board", "revision 0", timeout=30)
    keys("C-p", "C-u")
    literal("create task with product owner agent lens")
    expect("Create task with product owner agent lens")
    keys("Enter")
    expect("Task batch")
    literal("local-agent-task")
    keys("Enter")
    expect("Review 1 exact task(s)")
    keys("Enter")
    expect(
        "Open fresh Daimon session for Chronos task",
        "fresh session",
        "human gate",
        "does not submit the work prompt",
        timeout=20,
    )
    keys("c", "Enter")
    expect(
        "DAIMON",
        "Compose Daimon prompt",
        "product-owner collaboration lens",
        "not a human identity",
        "local-agent-task",
        timeout=20,
    )
    keys("Tab", "c", "Enter")
    expect("Daimon guarded session set", "prompt ready session")
    keys("c", "Enter")
    expect(
        "AGORA",
        "Create Agora room",
        "local-agent-task",
        "direct · local-agent, operator",
        timeout=20,
    )
    keys("c", "Enter")
    expect(
        "Bind Chronos task to Agora room",
        "local-agent-task",
        "human gate",
        timeout=20,
    )
    keys("c", "Enter")
    expect(
        "BOUND · Owner created and exact readback verified relationship",
        "Agora room local-agent-task durably created",
        "DONE · Local-agent turn accepted",
        timeout=20,
    )
    keys("Escape")
    keys("Escape")
    expect("Live Room", "Task · local-agent-task", timeout=20)
    keys("Down", "Enter")
    expect("Conversation · local-agent-task", timeout=20)
    keys("C-p", "C-u")
    literal("compose an agora message")
    expect("Compose an Agora message")
    keys("Enter")
    expect("Compose Agora message", "revision")
    literal("Please inspect the bounded acceptance evidence")
    keys("Tab")
    literal("local-agent")
    keys("Tab", "c", "Enter")
    expect("Agora accepted message #", timeout=20)
    keys("Escape")
    expect("Please inspect the bounded acceptance evidence", "to: local-agent", timeout=20)
    keys("C-p", "C-u")
    literal("offer addressed agora message to agent cohort")
    expect("Offer addressed Agora message to agent cohort")
    keys("Enter")
    expect(
        "Compose Daimon prompt",
        "ordinary room chatter is not a prompt",
        "Please inspect the bounded acceptance evidence",
        "local-agent-task",
        timeout=20,
    )
    keys("Tab", "c", "Enter")
    expect("Daimon guarded session set", "prompt ready session")
    keys("c", "Enter")
    expect(
        "Addressed message accepted",
        "Routed turns applied",
        "ambient chatter inert",
        timeout=20,
    )
    keys("C-p", "C-u")
    literal("return to the kanban board")
    expect("Return to the Kanban board")
    keys("Enter")
    expect("CHRONOS", "Board", "local-agent-task", timeout=20)
    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", session, check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after the settled journey")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

inspect_relationship
drain_link
start_link
inspect_relationship

test "$(wc -l <"$scratch/receipts" | tr -d ' ')" = 3
grep '^opened$' "$scratch/receipts" >/dev/null
test "$(grep -c '^prompted$' "$scratch/receipts")" = 2

echo 'PASS: Chronos task -> exact owner binding -> human-gated fresh Daimon session -> guarded prompt -> human-gated direct Agora task conversation -> durable exact relationship -> explicitly addressed room message -> guarded second agent turn -> refreshed Kanban board -> exact headless relationship inspection before/after owner restart'
