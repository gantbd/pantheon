#!/bin/sh
set -eu

if [ "${PANTHEON_LMSTUDIO_LIVE:-}" != 1 ]; then
  echo 'SKIP: set PANTHEON_LMSTUDIO_LIVE=1 to run the explicit local-model spend gate'
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pantheon-lmstudio-live.XXXXXX")
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

model_name=${PANTHEON_LMSTUDIO_MODEL:-qwen/qwen3-coder-next}
provider_address=${PANTHEON_LMSTUDIO_ADDRESS:-127.0.0.1:1234}
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
"$chronos" "$scratch/chronos.sock" >"$scratch/chronos.out" 2>"$scratch/chronos.err" &
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
  '  "aegis_grant_expires_at": 1000,' \
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
  '    "chronos_task_id": "local-model-task"' \
  '  }],' \
  '  "context": [{' \
  '    "reference": "17171717171717171717171717171717",' \
  '    "kind": "chronos_work",' \
  '    "text": "bounded LM Studio qualification"' \
  '  }],' \
  '  "messages": [{"role": "system", "content": "answer carefully"}],' \
  '  "io_timeout_ms": 300000,' \
  '  "max_executions": 8,' \
  '  "max_requests_per_connection": 4' \
  '}' >"$scratch/model.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "LM Studio local-model dogfood",' \
  '  "surfaces": ["daimon", "configuration"],' \
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
        capture_output=True, text=True, check=check, timeout=10,
    )

def expect(*markers, timeout=20):
    deadline = time.monotonic() + timeout
    screen = ""
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "lmstudio").stdout
        if all(marker in screen for marker in markers):
            return screen
        time.sleep(0.05)
    raise AssertionError(f"Missing {markers!r}:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "200", "-y", "38", "-s", "lmstudio", command)
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
assert report["hosted_shutdown"]["status"] == "clean"
PYTHON

kill -TERM "$chronos_pid"
wait "$chronos_pid" || true
chronos_pid=
kill -TERM "$aegis_pid"
wait "$aegis_pid"
aegis_pid=

echo "PASS: Pantheon guarded and retained live LM Studio qualification for $model_name"
