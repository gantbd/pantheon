#!/bin/sh
set -eu

if [ "${PANTHEON_LMSTUDIO_LIVE:-}" != 1 ]; then
  echo 'SKIP: set PANTHEON_LMSTUDIO_LIVE=1 to run the explicit local-model spend gate'
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
# Keep the composed Unix-domain topology below macOS's small socket-path limit.
scratch=$(mktemp -d "/private/tmp/plm.XXXXXX")
aegis_pid=
chronos_pid=

cleanup() {
  for pid in "$chronos_pid" "$aegis_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/workstream"
chmod 700 "$scratch/workstream"

model_name=${PANTHEON_LMSTUDIO_MODEL:-qwen/qwen3-coder-next}
provider_address=${PANTHEON_LMSTUDIO_ADDRESS:-127.0.0.1:1234}
review_context='Act as an independent architecture reviewer. Chronos public coordination supports submit and cancel, while claim and complete require a trusted in-process worker profile with native grants and leases. Pantheon work may be completed externally. Propose the smallest owner-safe contract for admitting evidence of external completion. Preserve grant, lease, revision and evidence semantics; do not expose worker authority as a bearer protocol; require named-human review. Separate facts, recommendation, risks and rejected shortcuts. Do not invent identifiers or claim APIs already exist.'
if [ -n "${PANTHEON_LMSTUDIO_REVIEW_CONTEXT_FILE:-}" ]; then
  review_context=$(python3 - "$PANTHEON_LMSTUDIO_REVIEW_CONTEXT_FILE" <<'PYTHON'
from pathlib import Path
import sys

path = Path(sys.argv[1])
payload = path.read_bytes()
if not payload or len(payload) > 65536:
    raise SystemExit("review context must contain 1..65536 bytes")
print(payload.decode("utf-8"), end="")
PYTHON
  )
fi
review_context_json=$(python3 - "$review_context" <<'PYTHON'
import json, sys
print(json.dumps(sys.argv[1]))
PYTHON
)
python3 - "$provider_address" "$model_name" <<'PYTHON'
import json, sys, urllib.request
address, selected = sys.argv[1:]
with urllib.request.urlopen(f"http://{address}/v1/models", timeout=3) as response:
    catalogue = json.load(response)
models = {entry["id"] for entry in catalogue.get("data", [])}
if selected not in models:
    raise SystemExit(f"selected LM Studio model is not loaded: {selected}")
PYTHON

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

"$aegis" create "$scratch/authority" "$scratch/aegis.sock" \
  11111111111111111111111111111111 "$uid" "$gid" \
  12121212121212121212121212121212 1313131313131313 administrator \
  >"$scratch/aegis.out" 2>"$scratch/aegis.err" &
aegis_pid=$!
"$chronos" "$scratch/chronos.sock" local-model-review >"$scratch/chronos.out" 2>"$scratch/chronos.err" &
chronos_pid=$!

attempt=0
while [ ! -S "$scratch/aegis.sock" ] || [ ! -S "$scratch/chronos.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$aegis_pid" 2>/dev/null || ! kill -0 "$chronos_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/aegis.err" >&2
    sed -n '1,160p' "$scratch/chronos.err" >&2
    echo 'FAIL: live owner prerequisites did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

printf '%s\n' \
  '{' \
  '  "principal": "12121212121212121212121212121212",' \
  "  \"aegis_socket\": \"$scratch/aegis.sock\"," \
  '  "aegis_expected_epoch": 0,' \
  '  "aegis_grant_now": 1,' \
  '  "aegis_grant_expires_at": 4000000000,' \
  "  \"chronos_socket\": \"$scratch/chronos.sock\"," \
  "  \"daimon_socket\": \"$scratch/model.sock\"," \
  "  \"store_directory\": \"$scratch/store\"," \
  '  "provider": {' \
  '    "provider_id": "14141414141414141414141414141414",' \
  '    "harness_profile_id": "15151515151515151515151515151515",' \
  "    \"model\": \"$model_name\"," \
  "    \"address\": \"$provider_address\"," \
  "    \"host_header\": \"$provider_address\"," \
  '    "path": "/v1/chat/completions",' \
  '    "structured_output": false,' \
  '    "structured_tools": true,' \
  '    "sampling_temperature_milli": 0,' \
  '    "connect_timeout_ms": 5000,' \
  '    "io_timeout_ms": 300000,' \
  '    "max_request_bytes": 65536,' \
  '    "max_response_bytes": 1048576' \
  '  },' \
  '  "task_bindings": [{' \
  '    "external_task_key": "16161616161616161616161616161616",' \
  '    "chronos_task_id": "local-model-review"' \
  '  }],' \
  '  "task_execution": {' \
  '    "task_incarnation": 1,' \
  '    "attempt_number": 1,' \
  '    "budget": {' \
  '      "max_elapsed_millis": 300000,' \
  '      "max_input_tokens": 8192,' \
  '      "max_output_tokens": 4096,' \
  '      "max_tool_calls": 1,' \
  '      "max_context_bytes": 65536,' \
  '      "max_output_bytes": 65536' \
  '    }' \
  '  },' \
  '  "context": [{' \
  '    "reference": "17171717171717171717171717171717",' \
  '    "kind": "chronos_work",' \
  "    \"text\": $review_context_json" \
  '  }],' \
  '  "messages": [{"role": "system", "content": "Be concise, explicit about uncertainty, and treat the supplied architecture statement as the complete evidence packet."}],' \
  '  "io_timeout_ms": 300000,' \
  '  "max_executions": 8,' \
  '  "max_requests_per_connection": 4' \
  '}' >"$scratch/model.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "LM Studio local-model dogfood",' \
  '  "surfaces": ["chronos", "daimon", "configuration"],' \
  '  "chronos": {' \
  '    "deployment": "hosted",' \
  "    \"workstreams\": [{\"name\": \"Local model review\", \"directory\": \"$scratch/workstream\"}]," \
  '    "requires": ["tasks.observe.v1"],' \
  '    "moira": [],' \
  '    "daimon": [],' \
  '    "planning": null' \
  '  },' \
  '  "daimon": {' \
  '    "deployment": "hosted",' \
  "    \"model_socket\": \"$scratch/model.sock\"," \
  '    "requires": ["models.endpoint.attach.v1", "models.provider.qualify.v1", "models.task.invoke.v1"],' \
  '    "host": {' \
  "      \"executable\": \"$model\"," \
  "      \"config\": \"$scratch/model.json\"" \
  '    }' \
  '  }' \
  '}' >"$scratch/profile.json"

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([
    binary, "interactive", "--workspace-profile", profile, "--chronos-allow-writes",
])
command += " 2>" + shlex.quote(str(Path(scratch) / "pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=10,
    )

def expect(*markers, timeout=20):
    deadline = time.monotonic() + timeout
    screen = ""
    while time.monotonic() < deadline:
        captured = tmux("capture-pane", "-p", "-t", "lmstudio", check=False)
        if captured.returncode != 0:
            error = Path(scratch, "pantheon.err").read_text(errors="replace")
            raise AssertionError(f"Pantheon exited before the expected screen:\n{error}")
        screen = captured.stdout
        if all(marker in screen for marker in markers):
            return screen
        time.sleep(0.05)
    raise AssertionError(f"Missing {markers!r}:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "200", "-y", "38", "-s", "lmstudio", command)
    expect("CHRONOS", "Local model review", "write-enabled")
    tmux("send-keys", "-t", "lmstudio", "C-p")
    tmux("send-keys", "-t", "lmstudio", "-l", "go to daimon")
    tmux("send-keys", "-t", "lmstudio", "Enter")
    expect("DAIMON", "MODEL ENDPOINT ATTACHED", "No execution, provider health, qualification")
    tmux("send-keys", "-t", "lmstudio", "C-p")
    tmux("send-keys", "-t", "lmstudio", "-l", "qualify selected local model provider")
    tmux("send-keys", "-t", "lmstudio", "Enter")
    expect("Qualify local model provider", "READY", "qwen/qwen3-coder-next")
    tmux("send-keys", "-t", "lmstudio", "c")
    expect("ARMED", "transport uncertainty will not be retried")
    tmux("send-keys", "-t", "lmstudio", "Enter")
    expect(
        "QUALIFIED",
        "qualified_completion",
        "tool continuation    not_exercised",
        "Boundary: completion qualification is not task invocation",
        timeout=320,
    )
    tmux("send-keys", "-t", "lmstudio", "C-g")
    tmux("send-keys", "-t", "lmstudio", "C-p", "C-u")
    tmux("send-keys", "-t", "lmstudio", "-l", "create task with local model")
    expect("Create task with local model")
    tmux("send-keys", "-t", "lmstudio", "Enter")
    expect("Task batch")
    tmux("send-keys", "-t", "lmstudio", "-l", "local-model-review")
    tmux("send-keys", "-t", "lmstudio", "Enter")
    expect("Review 1 exact task(s)")
    tmux("send-keys", "-t", "lmstudio", "Enter")
    expect(
        "Invoke local model for Chronos task",
        "local-model-review",
        "human gate            review",
        timeout=20,
    )
    tmux("send-keys", "-t", "lmstudio", "c", "Enter")
    expect(
        "COMPLETED",
        "MODEL RESPONSE",
        "Agora room and Chronos comment",
        "linkage remain open",
        timeout=320,
    )
    tmux("send-keys", "-t", "lmstudio", "C-g")
    tmux("send-keys", "-t", "lmstudio", "C-q")
    deadline = time.monotonic() + 15
    while tmux("has-session", "-t", "lmstudio", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not cleanly drain live LM Studio topology")
        time.sleep(0.05)
finally:
    tmux("kill-server", check=False)
PYTHON

headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" >"$scratch/headless.json"
python3 - "$model_name" "$scratch/headless.json" <<'PYTHON'
import json, sys
from pathlib import Path
selected, report_path = sys.argv[1:]
report = json.loads(Path(report_path).read_text(encoding="utf-8"))
assert report["complete"]
provider = report["provider_configuration"]["catalog"]["providers"][0]
assert provider["model"] == selected
qualification = provider["qualification"]
assert qualification["disposition"] == "qualified_completion"
assert qualification["completion"] is True
assert qualification["tool_continuation"] == "not_exercised"
assert qualification["input_tokens"] > 0
assert qualification["output_tokens"] > 0
assert qualification["output_bytes"] > 0
assert qualification["evidence_digest"].startswith("sha256:")
assert "output" not in qualification
invocation = report["provider_configuration"]["catalog"]["latest_task_invocation"]
assert invocation["chronos_task_id"] == "local-model-review"
assert invocation["disposition"] == "completed"
assert invocation["input_tokens"] > 0
assert invocation["output_tokens"] > 0
assert invocation["output_bytes"] > 0
assert invocation["output_digest"].startswith("sha256:")
assert "output" not in invocation
assert report["hosted_shutdown"]["status"] == "clean"
PYTHON

python3 - "$scratch" <<'PYTHON'
import json, sys
from pathlib import Path

records = list((Path(sys.argv[1]) / "store.task-invocations").glob("*.json"))
if len(records) != 1:
    raise SystemExit("expected one retained local-model review")
record = json.loads(records[0].read_text(encoding="utf-8"))
output = record.get("output", "")
if not output:
    raise SystemExit("local-model review output was empty")
print("LOCAL_MODEL_REVIEW_BEGIN")
print(output)
print("LOCAL_MODEL_REVIEW_END")
PYTHON

if kill -0 "$chronos_pid" 2>/dev/null; then
  kill -TERM "$chronos_pid"
fi
wait "$chronos_pid" || true
chronos_pid=
if kill -0 "$aegis_pid" 2>/dev/null; then
  kill -TERM "$aegis_pid"
fi
wait "$aegis_pid" || true
aegis_pid=

echo "PASS: Pantheon qualified and invoked $model_name for one reviewed Chronos task"
