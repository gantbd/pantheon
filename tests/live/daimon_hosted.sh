#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-daimon-hosted.XXXXXX")
aegis_pid=
agent_pid=
chronos_pid=

cleanup() {
  for pid in "$agent_pid" "$chronos_pid" "$aegis_pid"; do
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
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

aegis="$workspace/aegis/target/debug/aegis-daemon"
agent="$workspace/daimon/target/debug/daimon-local-agent"
chronos="$workspace/daimon/target/debug/examples/qualification_chronos"
seed="$workspace/daimon/target/debug/examples/qualification_seed"
workbench="$root/target/debug/pantheon"
provider_fixture="$workspace/daimon/adapters/daimon-acp/tests/support/fake_acp_agent.sh"
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

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted Daimon qualification",' \
  '  "surfaces": ["daimon"],' \
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
peer = report["peers"][0]
assert peer["peer"] == "daimon" and peer["status"] == "current"
assert peer["capability"] == "agents.session.inspect.v1"
session = peer["configured_agent_session"]
assert session["status"] == "current"
assert session["phase"] == "ReattachRequired" and session["revision"] == 1
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [{"peer": "daimon", "instance": None, "disposition": "clean"}],
}
'
[ ! -e "$scratch/agent.sock" ]

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

try:
    tmux("new-session", "-d", "-x", "130", "-y", "20", "-s", "daimon", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "daimon").stdout
        if "Daimon agent" in screen and "ReattachRequired/1" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Daimon pane not ready:\n{screen}")
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

kill -TERM "$aegis_pid"
wait "$aegis_pid"
aegis_pid=
[ ! -e "$scratch/aegis.sock" ]

echo 'PASS: production Daimon agent -> durable session -> Pantheon hosted headless and interactive reopen -> clean drain'
