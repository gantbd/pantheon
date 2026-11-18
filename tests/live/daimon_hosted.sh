#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-hosted.XXXXXX")
aegis_pid=
agent_pid=
chronos_pid=
vellum_pid=

cleanup() {
  for pid in "$agent_pid" "$chronos_pid" "$vellum_pid" "$aegis_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/aegis/Cargo.toml" \
  --package aegis-daemon --bin aegis-daemon --locked --offline
cargo build --manifest-path "$workspace/daimon/Cargo.toml" \
  --package daimon-local-agent --bin daimon-local-agent \
  --example qualification_chronos --example qualification_seed --locked --offline
cargo build --manifest-path "$workspace/vellum/Cargo.toml" \
  --package vellum-daemon --bin vellum-daemon --locked --offline \
  --target-dir "$root/target"
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

aegis="$workspace/aegis/target/debug/aegis-daemon"
agent="$workspace/daimon/target/debug/daimon-local-agent"
chronos="$workspace/daimon/target/debug/examples/qualification_chronos"
seed="$workspace/daimon/target/debug/examples/qualification_seed"
vellum_daemon="$root/target/debug/vellum-daemon"
workbench="$root/target/debug/pantheon"
provider_fixture="$workspace/daimon/adapters/daimon-acp/tests/support/fake_acp_agent.sh"
uid=$(id -u)
gid=$(id -g)

printf 'exact Vellum revision' >"$scratch/context.txt"
"$vellum_daemon" --provision-credential "$scratch/vellum.credential"
"$vellum_daemon" --socket "$scratch/vellum.sock" --control "$scratch/vellum.control" \
  --credential "$scratch/vellum.credential" -- "$scratch/context.txt" \
  >"$scratch/vellum.out" 2>"$scratch/vellum.err" &
vellum_pid=$!
attempt=0
while [ ! -S "$scratch/vellum.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$vellum_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/vellum.err" >&2
    echo 'FAIL: Vellum context owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

"$aegis" create "$scratch/authority" "$scratch/aegis.sock" \
  01010101010101010101010101010101 "$uid" "$gid" \
  02020202020202020202020202020202 0303030303030303 administrator \
  >"$scratch/aegis.out" 2>"$scratch/aegis.err" &
aegis_pid=$!

attempt=0
while [ ! -S "$scratch/aegis.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$aegis_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/aegis.err" >&2
    echo 'FAIL: Aegis owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

printf '%s\n' \
  '{' \
  '  "principal": "02020202020202020202020202020202",' \
  "  \"aegis_socket\": \"$scratch/aegis.sock\"," \
  '  "aegis_expected_epoch": 0,' \
  '  "aegis_grant_now": 1,' \
  '  "aegis_grant_expires_at": 1000,' \
  "  \"chronos_socket\": \"$scratch/chronos.sock\"," \
  "  \"daimon_socket\": \"$scratch/agent.sock\"," \
  "  \"store_directory\": \"$scratch/store\"," \
  '  "providers": [{' \
  '    "provider_id": "02020202020202020202020202020202",' \
  '    "harness_profile_id": "03030303030303030303030303030303",' \
  '    "policy_digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222",' \
  '    "program": "/bin/sh",' \
  "    \"arguments\": [\"$provider_fixture\", \"resume\"]," \
  '    "inherited_environment_names": [],' \
  '    "isolation": {"kind": "direct"},' \
  '    "transport": "acp"' \
  '  }],' \
  '  "workspaces": [{' \
  '    "identity": "06060606060606060606060606060606",' \
  "    \"path\": \"$scratch\"" \
  '  }],' \
  '  "task_bindings": [{' \
  '    "external_task_key": "0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",' \
  '    "chronos_task_id": "local-agent-task"' \
  '  }],' \
  '  "tool_capabilities": [],' \
  '  "context": [],' \
  '  "vellum_context": [{' \
  '    "reference": "08080808080808080808080808080808",' \
  "    \"socket\": \"$scratch/vellum.sock\"," \
  "    \"credential\": \"$scratch/vellum.credential\"," \
  '    "workspace_id": 1,' \
  '    "document_ordinal": 1,' \
  '    "expected_revision": 0' \
  '  }],' \
  '  "io_timeout_ms": 3000,' \
  '  "max_agent_sessions": 8,' \
  '  "max_service_sessions": 4,' \
  '  "max_requests_per_connection": 16,' \
  '  "max_store_bytes": 1048576,' \
  '  "max_context_bytes": 4096' \
  '}' >"$scratch/agent.json"

"$chronos" "$scratch/chronos.sock" >"$scratch/chronos.out" 2>"$scratch/chronos.err" &
chronos_pid=$!
mkfifo "$scratch/control"
exec 3<>"$scratch/control"
"$agent" --supervised "$scratch/agent.json" <"$scratch/control" \
  >"$scratch/agent.out" 2>"$scratch/agent.err" &
agent_pid=$!
attempt=0
while [ ! -S "$scratch/agent.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$agent_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/agent.err" >&2
    echo 'FAIL: Daimon seed owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

revision=$("$seed" "$scratch/agent.sock")
[ "$revision" = 1 ]
printf x >&3
exec 3>&-
wait "$agent_pid"
agent_pid=
wait "$chronos_pid"
chronos_pid=
[ ! -e "$scratch/agent.sock" ]
"$vellum_daemon" --shutdown "$scratch/vellum.control" >/dev/null
wait "$vellum_pid"
vellum_pid=
[ ! -e "$scratch/vellum.sock" ]

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted Daimon qualification",' \
  '  "surfaces": ["daimon", "vellum"],' \
  '  "vellum": {' \
  '    "deployment": "hosted",' \
  "    \"socket\": \"$scratch/vellum.sock\"," \
  "    \"credential\": \"$scratch/vellum.credential\"," \
  "    \"roots\": [\"$scratch/context.txt\"]," \
  '    "requires": ["workspace.inventory.v1"]' \
  '  },' \
  '  "daimon": {' \
  '    "deployment": "hosted",' \
  "    \"agent_socket\": \"$scratch/agent.sock\"," \
  '    "agent_session": "07070707070707070707070707070707",' \
  '    "requires": ["agents.session.inspect.v1"],' \
  '    "host": {' \
  "      \"executable\": \"$agent\"," \
  "      \"config\": \"$scratch/agent.json\"" \
  '    }' \
  '  }' \
  '}' >"$scratch/profile.json"

headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = next(peer for peer in report["peers"] if peer["peer"] == "daimon")
assert peer["peer"] == "daimon" and peer["status"] == "current"
assert peer["capability"] == "agents.session.inspect.v1"
session = peer["configured_agent_session"]
assert session["status"] == "current"
assert session["phase"] == "ReattachRequired" and session["revision"] == 1
context = session["last_context"]
assert context["execution"] == "0c" * 16
assert len(context["entries"]) == 1
entry = context["entries"][0]
assert entry == {
    "reference": "08" * 16,
    "kind": "vellum_document",
    "source_digest": "sha256:d0726d2d048d7c9ecb8d80f31dbded527ccde6e144141ddc4d215391e149e346",
    "content_bytes": 21,
    "tellus": None,
}
assert context["content_disclosure"] == "not_published"
vellum = next(peer for peer in report["peers"] if peer["peer"] == "vellum")
assert vellum["status"] == "current"
assert vellum["context_manifest"] == [{
    "document": 1,
    "revision": 0,
    "witness": 1,
    "bytes": 21,
    "sha256": entry["source_digest"],
}]
assert vellum["content_disclosure"] == "not_published"
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [
        {"peer": "daimon", "instance": None, "disposition": "clean"},
        {"peer": "vellum", "instance": None, "disposition": "clean"},
    ],
}
'
[ ! -e "$scratch/agent.sock" ]

python3 - "$scratch" "$workbench" "$provider_fixture" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary, provider_fixture = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

try:
    tmux("new-session", "-d", "-x", "220", "-y", "20", "-s", "daimon", command)
    tmux("send-keys", "-t", "daimon", "M-5")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if all(marker in screen for marker in (
            "Configured sessions",
            "ReattachRequired/1",
            "last context: execution 0c0c0c0c",
            "VellumDocument",
            "21 bytes",
        )):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Daimon pane not ready:\n{screen}")
    tmux("send-keys", "-t", "daimon", "C-p")
    tmux("send-keys", "-t", "daimon", "-l", "inspect local provider configuration")
    tmux("send-keys", "-t", "daimon", "Enter")
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if all(marker in screen for marker in (
            "Local provider configuration",
            "Daimon validated the selected owner document",
            "daimon-local-agent",
            "agent transport",
            "acp",
            "commands, arguments, paths, credential names, and secrets remain redacted",
        )):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Provider catalogue not ready:\n{screen}")
    tmux("send-keys", "-t", "daimon", "r")
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if "REFRESHED" in screen and "CURRENT · Daimon validated" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Provider catalogue refresh did not settle:\n{screen}")
    tmux("send-keys", "-t", "daimon", "C-g")
    tmux("send-keys", "-t", "daimon", "C-p")
    tmux("send-keys", "-t", "daimon", "-l", "add or replace a local agent provider")
    tmux("send-keys", "-t", "daimon", "Enter")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if "Add or replace local agent provider" in screen and "configuration only" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Provider edit form did not open:\n{screen}")
    for value in (
        "02020202020202020202020202020202",
        "03030303030303030303030303030303",
    ):
        tmux("send-keys", "-t", "daimon", "C-u")
        tmux("send-keys", "-t", "daimon", "-l", value)
        tmux("send-keys", "-t", "daimon", "Tab")
    tmux("send-keys", "-t", "daimon", "-l", "sha256:" + "99" * 32)
    tmux("send-keys", "-t", "daimon", "Tab")
    tmux("send-keys", "-t", "daimon", "-l", "/bin/sh")
    tmux("send-keys", "-t", "daimon", "Tab")
    tmux("send-keys", "-t", "daimon", "-l", provider_fixture)
    for _ in range(6):
        tmux("send-keys", "-t", "daimon", "Tab")
    tmux("send-keys", "-t", "daimon", "Enter")
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if all(marker in screen for marker in (
            "REVIEWING",
            "candidate document",
            "confirmation plan",
            "policy_digest",
            "arguments",
            "configuration only",
        )):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Provider edit plan did not settle:\n{screen}")
    tmux("send-keys", "-t", "daimon", "c")
    tmux("send-keys", "-t", "daimon", "Enter")
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if all(marker in screen for marker in (
            "APPLIED",
            "restart required",
            "no qualification",
            "action              replace",
        )):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Provider edit apply did not settle:\n{screen}")
    tmux("send-keys", "-t", "daimon", "C-g")
    tmux("send-keys", "-t", "daimon", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "daimon", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon did not drain hosted Daimon")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/agent.sock" ]
python3 - "$scratch/agent.json" <<'PYTHON'
import json, sys
with open(sys.argv[1], encoding="utf-8") as source:
    provider = json.load(source)["providers"][0]
assert provider["policy_digest"] == "sha256:" + "99" * 32
assert provider["arguments"] and provider["arguments"][0].endswith("fake_acp_agent.sh")
PYTHON

kill -TERM "$aegis_pid"
wait "$aegis_pid"
aegis_pid=
[ ! -e "$scratch/aegis.sock" ]

echo 'PASS: production Daimon agent -> redacted catalogue -> guarded plan/apply provider edit -> durable session -> Pantheon hosted reopen -> clean drain'
