#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
# Unix-domain endpoints have a small path ceiling on macOS; keep the complete
# Chronos/Daimon fixture topology under the short private temp root.
scratch=$(mktemp -d "/private/tmp/pwiaj.XXXXXX")
fixture_pid=

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/chronos"
chmod 700 "$scratch/chronos"

cargo build --manifest-path "$workspace/daimon-tui/Cargo.toml" \
  --package daimon-tui-cli --example agent_open_journey_fixture --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

workbench="$root/target/debug/pantheon"
catalog_owner="$root/tests/support/provider_catalog_open_owner.sh"

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
  "surfaces": ["chronos", "daimon"],
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
        "Local-agent turn accepted",
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

test "$(wc -l <"$scratch/receipts" | tr -d ' ')" = 2
grep '^opened$' "$scratch/receipts" >/dev/null
grep '^prompted$' "$scratch/receipts" >/dev/null

echo 'PASS: Chronos task -> exact owner binding -> human-gated fresh Daimon session -> guarded product-owner-lensed prompt -> refreshed Kanban board'
