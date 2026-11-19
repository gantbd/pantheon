#!/bin/sh
set -eu

if [ "${PANTHEON_LIVE_MODEL:-0}" != 1 ]; then
  echo 'SKIP: set PANTHEON_LIVE_MODEL=1 to download and run the governed real-model gate'
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/phr.XXXXXX")
hestia_pid=

cleanup() {
  if [ -n "$hestia_pid" ] && kill -0 "$hestia_pid" 2>/dev/null; then
    kill "$hestia_pid" 2>/dev/null || true
    wait "$hestia_pid" 2>/dev/null || true
  fi
  if [ "${PANTHEON_KEEP_REAL_MODEL:-0}" = 1 ]; then
    echo "Retained real-model gate evidence at $scratch"
  else
    rm -rf "$scratch"
  fi
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/artifacts"
chmod 700 "$scratch/artifacts"

llama_server=${PANTHEON_LLAMA_SERVER:-/opt/homebrew/bin/llama-server}
if [ ! -x "$llama_server" ]; then
  echo "FAIL: llama-server is unavailable at $llama_server" >&2
  exit 1
fi
model_source=${PANTHEON_REAL_MODEL_SOURCE:-}
if [ -n "$model_source" ] && [ ! -f "$model_source" ]; then
  echo "FAIL: PANTHEON_REAL_MODEL_SOURCE is not a regular file: $model_source" >&2
  exit 1
fi

cargo build --manifest-path "$workspace/hestia/Cargo.toml" \
  --package hestia-daemon --bin hestia-daemon \
  --package hestiactl --bin hestiactl --locked --offline

hestia="$workspace/hestia/target/debug/hestia-daemon"
hestiactl="$workspace/hestia/target/debug/hestiactl"
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')

python3 - "$scratch" "$llama_server" "$port" "$model_source" <<'PYTHON'
from pathlib import Path
import json, sys

scratch, llama_server, port, local_source = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), sys.argv[4]
digest = bytes.fromhex("cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046")
model = {
    "identity": "qwen/qwen2.5-coder-1.5b-instruct",
    "digest": list(digest),
    "quantization": "Q4_K_M",
    "context_tokens": 4096,
}
configuration = {
    "version": 1,
    "socket": str(scratch / "hestia.sock"),
    "authority": [12] * 16,
    "max_runtimes": 1,
    "profiles": [{
        "id": "qwen-coder-small",
        "endpoint_reference": "qwen-coder-small-loopback",
        "executable": llama_server,
        "arguments": [
            "--host", "127.0.0.1", "--port", str(port),
            "--ctx-size", "4096", "--parallel", "1",
            "--alias", model["identity"], "--no-webui", "--model",
        ],
        "artifact": {"artifact": "qwen-coder-small-q4", "argument_index": 12},
        "environment": [],
        "readiness_address": f"127.0.0.1:{port}",
        "startup_timeout_ms": 55000,
        "probe_interval_ms": 100,
        "drain_timeout_ms": 10000,
    }],
    "artifacts": {
        "directory": str(scratch / "artifacts"),
        "max_inflight": 1,
        "entries": [{
            "entry": {
                "artifact": "qwen-coder-small-q4",
                "model": model,
                "registry": "huggingface/Qwen",
                "source_revision": "2ab9f8f42af02fc212effaef7c4850c885e965f4",
                "license": "Apache-2.0",
                "format": "Gguf",
                "expected_size_bytes": 1117320768,
                "expected_digest": list(digest),
            },
            "executable": "/bin/cat" if local_source else "/usr/bin/curl",
            "arguments": [] if local_source else ["-fL", "--retry", "3", "--connect-timeout", "30", "--silent", "--show-error"],
            "source": local_source or "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/2ab9f8f42af02fc212effaef7c4850c885e965f4/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
        }],
    },
}
acquire = {
    "authority": [12] * 16,
    "artifact": "qwen-coder-small-q4",
    "operation": [13] * 16,
    "expected_revision": 1,
    "accepted_license": "Apache-2.0",
    "expected_digest": list(digest),
}
start = {
    "authority": [12] * 16,
    "runtime": [14] * 16,
    "operation": [15] * 16,
    "expected_revision": 0,
    "expected_incarnation": 0,
    "hosting_profile": "qwen-coder-small",
    "model": model,
    "resources": {
        "memory_bytes": 3 * 1024 * 1024 * 1024,
        "accelerator": "metal",
        "threads": 8,
        "concurrency": 1,
    },
    "sandbox": "Inherited",
    "egress": "Inherited",
}
for name, document in (("hestia.json", configuration), ("acquire.json", acquire), ("start.json", start)):
    (scratch / name).write_text(json.dumps(document))
PYTHON
chmod 600 "$scratch/hestia.json" "$scratch/acquire.json" "$scratch/start.json"

"$hestia" "$scratch/hestia.json" >"$scratch/hestia.out" 2>"$scratch/hestia.err" &
hestia_pid=$!
attempt=0
while [ ! -S "$scratch/hestia.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$hestia_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/hestia.err" >&2
    echo 'FAIL: Hestia owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

echo 'Acquiring Qwen2.5-Coder-1.5B-Instruct Q4_K_M through Hestia (1,117,320,768 bytes)'
"$hestiactl" acquire "$scratch/hestia.sock" "$scratch/acquire.json" >/dev/null
attempt=0
while :; do
  artifact=$("$hestiactl" artifact "$scratch/hestia.sock" qwen-coder-small-q4)
  lifecycle=$(printf '%s' "$artifact" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["lifecycle"])')
  observed=$(printf '%s' "$artifact" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["observed_bytes"])')
  if [ "$lifecycle" = Admitted ]; then
    echo "Artifact admitted after $observed bytes"
    break
  fi
  if [ "$lifecycle" = Failed ]; then
    printf '%s\n' "$artifact" >&2
    echo 'FAIL: Hestia artifact acquisition failed' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  if [ $((attempt % 10)) -eq 0 ]; then
    echo "  acquired $observed / 1117320768 bytes"
  fi
  if [ "$attempt" -ge 2400 ]; then
    echo 'FAIL: Hestia artifact acquisition timed out' >&2
    exit 1
  fi
  sleep 0.5
done

echo 'Starting the admitted artifact under Hestia-owned llama-server'
"$hestiactl" start "$scratch/hestia.sock" "$scratch/start.json" >"$scratch/start.out"
python3 - "$scratch/start.out" "$scratch/drain.json" <<'PYTHON'
from pathlib import Path
import json, sys

start = json.loads(Path(sys.argv[1]).read_text())
ready = start["result"]["Applied"]
assert ready["lifecycle"] == "Ready", start
Path(sys.argv[2]).write_text(json.dumps({
    "authority": [12] * 16,
    "runtime": [14] * 16,
    "operation": [16] * 16,
    "expected_revision": ready["revision"],
    "expected_incarnation": ready["incarnation"],
}))
PYTHON
chmod 600 "$scratch/drain.json"

python3 - "$port" <<'PYTHON'
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError
import json, sys, time

port = int(sys.argv[1])
base = f"http://127.0.0.1:{port}"
deadline = time.monotonic() + 180
while True:
    try:
        with urlopen(base + "/v1/models", timeout=10) as response:
            models = json.load(response)
        break
    except (HTTPError, URLError, TimeoutError):
        if time.monotonic() >= deadline:
            raise
        time.sleep(0.5)
assert any(item["id"] == "qwen/qwen2.5-coder-1.5b-instruct" for item in models["data"]), models
request = Request(
    base + "/v1/chat/completions",
    data=json.dumps({
        "model": "qwen/qwen2.5-coder-1.5b-instruct",
        "messages": [{"role": "user", "content": "Reply with a short greeting."}],
        "temperature": 0,
        "max_tokens": 24,
    }).encode(),
    headers={"content-type": "application/json"},
)
with urlopen(request, timeout=60) as response:
    completion = json.load(response)
text = completion["choices"][0]["message"]["content"].strip()
assert text, completion
assert completion["usage"]["completion_tokens"] > 0, completion
print("Model response:", text.replace("\n", " ")[:160])
PYTHON

if [ "${PANTHEON_REAL_MODEL_DIAGNOSTIC:-0}" = 1 ]; then
  echo 'Qualifying Daimon structured-output diagnostics against the Hestia-owned endpoint'
  cargo build --manifest-path "$workspace/daimon/Cargo.toml" \
    --package daimon-local-model --bin daimon-local-model --locked --offline
  diagnostic_model="$workspace/daimon/target/debug/daimon-local-model"
  python3 - "$scratch" "$port" <<'PYTHON'
from pathlib import Path
import json, sys

scratch, port = Path(sys.argv[1]), int(sys.argv[2])
config = {
    "principal": "01010101010101010101010101010101",
    "aegis_socket": str(scratch / "diagnostic-aegis.sock"),
    "aegis_expected_epoch": 0,
    "aegis_grant_now": 1,
    "aegis_grant_expires_at": 1000,
    "chronos_socket": str(scratch / "diagnostic-chronos.sock"),
    "daimon_socket": str(scratch / "diagnostic-daimon.sock"),
    "store_directory": str(scratch / "diagnostic-store"),
    "provider": {
        "provider_id": "02020202020202020202020202020202",
        "harness_profile_id": "03030303030303030303030303030303",
        "model": "qwen/qwen2.5-coder-1.5b-instruct",
        "address": f"127.0.0.1:{port}",
        "host_header": f"127.0.0.1:{port}",
        "path": "/v1/chat/completions",
        "structured_output": True,
        "structured_tools": False,
        "sampling_temperature_milli": 0,
        "connect_timeout_ms": 5000,
        "io_timeout_ms": 300000,
        "max_request_bytes": 65536,
        "max_response_bytes": 1048576,
    },
    "task_bindings": [{
        "external_task_key": "04040404040404040404040404040404",
        "chronos_task_id": "structured-output-diagnostic",
    }],
    "context": [{
        "reference": "05050505050505050505050505050505",
        "kind": "chronos_work",
        "text": "diagnose the selected structured-output profile only",
    }],
    "messages": [{"role": "system", "content": "Return bounded output only."}],
    "io_timeout_ms": 300000,
    "max_executions": 2,
    "max_requests_per_connection": 2,
}
(scratch / "diagnostic-model.json").write_text(json.dumps(config))
PYTHON
  "$diagnostic_model" --describe-providers "$scratch/diagnostic-model.json" \
    >"$scratch/diagnostic-catalog.json"
  python3 - "$scratch/diagnostic-catalog.json" "$scratch/diagnostic-request.json" <<'PYTHON'
from pathlib import Path
import json, sys

catalog = json.loads(Path(sys.argv[1]).read_text())
Path(sys.argv[2]).write_text(json.dumps({
    "version": 1,
    "operation_id": "06060606060606060606060606060606",
    "expected_document_digest": catalog["document_digest"],
}))
PYTHON
  "$diagnostic_model" --qualify-provider "$scratch/diagnostic-model.json" \
    <"$scratch/diagnostic-request.json" >"$scratch/diagnostic-result.json"
  python3 - "$scratch/diagnostic-result.json" <<'PYTHON'
from pathlib import Path
import json, sys

result = json.loads(Path(sys.argv[1]).read_text())
assert result["disposition"] == "outcome_unknown", result
assert result["completion"] is False, result
assert (
    result["diagnostic_kind"] == "invalid_structured_output"
    or result["diagnostic_kind"].startswith("http_status:")
), result
assert result["diagnostic_response_digest"].startswith("sha256:"), result
assert any("does not narrow outcome uncertainty" in item for item in result["known_deviations"]), result
for forbidden in ("error", "secret", "unsupported"):
    assert forbidden not in json.dumps(result).lower(), result
print("Daimon structured diagnostic:", result["diagnostic_kind"], result["diagnostic_response_digest"])
PYTHON
fi

if [ "${PANTHEON_REAL_MODEL_DAIMON:-0}" = 1 ]; then
  echo 'Composing the Hestia-owned endpoint through Daimon and Pantheon'
  PANTHEON_LOCAL_MODEL_LIVE=1 \
  PANTHEON_LOCAL_MODEL_LABEL='Hestia-owned llama-server' \
  PANTHEON_LOCAL_MODEL_NAME='qwen/qwen2.5-coder-1.5b-instruct' \
  PANTHEON_LOCAL_MODEL_ADDRESS="127.0.0.1:$port" \
  PANTHEON_LOCAL_MODEL_HESTIA_SOCKET="$scratch/hestia.sock" \
  PANTHEON_LOCAL_MODEL_HESTIA_RUNTIME='0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e' \
    "$root/tests/live/daimon_model_lmstudio.sh"
fi

"$hestiactl" drain "$scratch/hestia.sock" "$scratch/drain.json" >"$scratch/drain.out"
grep -q '"Stopped"' "$scratch/drain.out"
"$hestiactl" shutdown "$scratch/hestia.sock" 0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c >/dev/null
wait "$hestia_pid"
hestia_pid=

echo 'PASS: Hestia acquired, verified, served, exercised, drained, and shut down a real Qwen GGUF'
