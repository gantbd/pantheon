#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pantheon-daimon-model-hosted.XXXXXX")
aegis_pid=
chronos_pid=
provider_pid=

cleanup() {
  for pid in "$provider_pid" "$chronos_pid" "$aegis_pid"; do
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
  --package daimon-local-model --bin daimon-local-model \
  --package daimon-local-agent --example qualification_chronos --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

aegis="$workspace/aegis/target/debug/aegis-daemon"
model="$workspace/daimon/target/debug/daimon-local-model"
chronos="$workspace/daimon/target/debug/examples/qualification_chronos"
workbench="$root/target/debug/pantheon"
uid=$(id -u)
gid=$(id -g)

python3 "$root/tests/fixtures/openai_qualification_server.py" \
  "$scratch/provider.port" "$scratch/provider.count" \
  >"$scratch/provider.out" 2>"$scratch/provider.err" &
provider_pid=$!
attempt=0
while [ ! -s "$scratch/provider.port" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$provider_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/provider.err" >&2
    echo 'FAIL: local provider fixture did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done
provider_port=$(cat "$scratch/provider.port")

"$aegis" create "$scratch/authority" "$scratch/aegis.sock" \
  01010101010101010101010101010101 "$uid" "$gid" \
  02020202020202020202020202020202 0303030303030303 administrator \
  >"$scratch/aegis.out" 2>"$scratch/aegis.err" &
aegis_pid=$!
"$chronos" "$scratch/chronos.sock" >"$scratch/chronos.out" 2>"$scratch/chronos.err" &
chronos_pid=$!

attempt=0
while [ ! -S "$scratch/aegis.sock" ] || [ ! -S "$scratch/chronos.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$aegis_pid" 2>/dev/null || ! kill -0 "$chronos_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/aegis.err" >&2
    sed -n '1,160p' "$scratch/chronos.err" >&2
    echo 'FAIL: owner prerequisites did not become ready' >&2
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
  "  \"daimon_socket\": \"$scratch/model.sock\"," \
  "  \"store_directory\": \"$scratch/store\"," \
  '  "provider": {' \
  '    "provider_id": "04040404040404040404040404040404",' \
  '    "harness_profile_id": "05050505050505050505050505050505",' \
  '    "model": "qwen/qwen3-coder-next",' \
  "    \"address\": \"127.0.0.1:$provider_port\"," \
  "    \"host_header\": \"localhost:$provider_port\"," \
  '    "path": "/v1/chat/completions",' \
  '    "structured_output": false,' \
  '    "structured_tools": true,' \
  '    "sampling_temperature_milli": 0,' \
  '    "connect_timeout_ms": 100,' \
  '    "io_timeout_ms": 1000,' \
  '    "max_request_bytes": 65536,' \
  '    "max_response_bytes": 65536' \
  '  },' \
  '  "task_bindings": [{' \
  '    "external_task_key": "06060606060606060606060606060606",' \
  '    "chronos_task_id": "local-model-task"' \
  '  }],' \
  '  "context": [{' \
  '    "reference": "07070707070707070707070707070707",' \
  '    "kind": "chronos_work",' \
  '    "text": "bounded local-model work"' \
  '  }],' \
  '  "messages": [{"role": "system", "content": "answer carefully"}],' \
  '  "io_timeout_ms": 1000,' \
  '  "max_executions": 8,' \
  '  "max_requests_per_connection": 4' \
  '}' >"$scratch/model.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted local model qualification",' \
  '  "surfaces": ["daimon"],' \
  '  "daimon": {' \
  '    "deployment": "hosted",' \
  "    \"model_socket\": \"$scratch/model.sock\"," \
  '    "requires": ["models.endpoint.attach.v1", "models.provider.qualify.v1"],' \
  '    "host": {' \
  "      \"executable\": \"$model\"," \
  "      \"config\": \"$scratch/model.json\"" \
  '    }' \
  '  }' \
  '}' >"$scratch/profile.json"

set +e
headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
headless_status=$?
set -e
if [ "$headless_status" -ne 0 ]; then
  printf '%s\n' "$headless" >&2
  echo 'FAIL: hosted local-model headless observation was incomplete' >&2
  exit 1
fi
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = next(peer for peer in report["peers"] if peer["peer"] == "daimon")
assert peer["status"] == "current"
assert peer["capability"] == "models.endpoint.attach.v1"
assert peer["model_endpoint"]["status"] == "attached"
assert "no execution, provider health, qualification, or authority" in peer["model_endpoint"]["boundary"]
configuration = report["provider_configuration"]
assert configuration["status"] == "current"
catalog = configuration["catalog"]
assert catalog["owner"] == "daimon-local-model"
provider = catalog["providers"][0]
assert provider["model"] == "qwen/qwen3-coder-next"
assert provider["wire"] == "openai_chat_completions"
assert provider["endpoint_scope"] == "loopback" and provider["endpoint_port"] > 0
assert not provider["structured_output"] and provider["structured_tools"]
assert provider.get("qualification") is None
assert "address" not in provider and "path" not in provider and "bearer_token_env" not in provider
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [{"peer": "daimon", "instance": None, "disposition": "clean"}],
}
'
[ ! -e "$scratch/model.sock" ]

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

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    screen = ""
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "model").stdout
        if all(marker in screen for marker in markers):
            return screen
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r}:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "200", "-y", "32", "-s", "model", command)
    expect("DAIMON", "MODEL ENDPOINT ATTACHED", "No execution, provider health, qualification", timeout=20)
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "inspect local provider configuration")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "Local provider configuration",
        "daimon-local-model",
        "qwen/qwen3-coder-next",
        "openai_chat_completions",
        "loopback:",
        "commands, arguments, paths, credential names, and secrets remain redacted",
    )
    tmux("send-keys", "-t", "model", "C-g")
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "qualify selected local model provider")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "Qualify local model provider",
        "READY",
        "human gate          review",
        "tool-continuation conformance",
    )
    tmux("send-keys", "-t", "model", "c")
    expect("ARMED", "one bounded completion-conformance request")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "QUALIFIED",
        "qualified_completion",
        "17 input",
        "4 output tokens",
        "tool continuation    not_exercised",
        timeout=20,
    )
    tmux("send-keys", "-t", "model", "C-g")
    tmux("send-keys", "-t", "model", "C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "model", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not cleanly drain hosted local-model Daimon")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/model.sock" ]

headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
provider = report["provider_configuration"]["catalog"]["providers"][0]
qualification = provider["qualification"]
assert qualification["disposition"] == "qualified_completion"
assert qualification["completion"] is True
assert qualification["tool_continuation"] == "not_exercised"
assert qualification["input_tokens"] == 17
assert qualification["output_tokens"] == 4
assert qualification["output_bytes"] == 30
assert qualification["evidence_digest"].startswith("sha256:")
assert qualification["known_deviations"] == ["tool_continuation_not_exercised"]
assert report["hosted_shutdown"]["status"] == "clean"
'
[ "$(cat "$scratch/provider.count")" = 1 ]
[ ! -e "$scratch/model.sock" ]

kill -TERM "$chronos_pid"
wait "$chronos_pid" || true
chronos_pid=
kill -TERM "$aegis_pid"
wait "$aegis_pid"
aegis_pid=
kill -TERM "$provider_pid"
wait "$provider_pid" || true
provider_pid=

echo 'PASS: hosted local-model Daimon -> guarded owner-retained qualification -> headless/TUI receipt parity -> one provider call -> clean drain'
