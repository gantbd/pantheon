#!/bin/sh
set -eu

if [ "${PANTHEON_LOCAL_MODEL_ENSEMBLE:-0}" != 1 ]; then
  echo 'SKIP: set PANTHEON_LOCAL_MODEL_ENSEMBLE=1 to exercise the heterogeneous local Moira ensemble'
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
store=${PANTHEON_MODEL_ENSEMBLE_STORE:-/private/tmp/pantheon-local-model-bench-qualified/artifacts}
qwen_source=${PANTHEON_QWEN_ARTIFACT:-/private/tmp/phr.Ml8WLm/artifacts/cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046}
context=${PANTHEON_ENSEMBLE_CONTEXT_FILE:-$workspace/hestia/src/artifact.rs}
context_ranges=${PANTHEON_ENSEMBLE_CONTEXT_LINE_RANGES:-128-180,300-455,690-753}
dendrite_port=${PANTHEON_DENDRITE_PORT:-4498}
scratch=$(mktemp -d "/private/tmp/pme.XXXXXX")
hestia_pid=

cleanup() {
  if [ -n "$hestia_pid" ] && kill -0 "$hestia_pid" 2>/dev/null; then
    kill "$hestia_pid" 2>/dev/null || true
    wait "$hestia_pid" 2>/dev/null || true
  fi
  if [ "${PANTHEON_KEEP_LOCAL_MODEL_ENSEMBLE:-0}" = 1 ]; then
    echo "Retained local-ensemble evidence at $scratch"
  else
    rm -rf "$scratch"
  fi
}
trap cleanup EXIT HUP INT TERM

test -d "$store" || { echo "FAIL: qualified Hestia artifact store is unavailable: $store" >&2; exit 1; }
test -f "$qwen_source" || { echo "FAIL: retained Qwen artifact is unavailable: $qwen_source" >&2; exit 1; }
test -f "$context" || { echo "FAIL: bounded evidence packet is unavailable: $context" >&2; exit 1; }
context_bytes=$(wc -c <"$context" | tr -d ' ')
if [ "$context_bytes" -eq 0 ] || [ "$context_bytes" -gt 65536 ]; then
  echo "FAIL: evidence packet must contain 1..65536 bytes: $context" >&2
  exit 1
fi
mkdir -p "$scratch/runtime"
chmod 700 "$scratch" "$scratch/runtime"

llama_server=${PANTHEON_LLAMA_SERVER:-/opt/homebrew/bin/llama-server}
test -x "$llama_server" || { echo "FAIL: llama-server is unavailable at $llama_server" >&2; exit 1; }

cargo build --manifest-path "$workspace/hestia/Cargo.toml" \
  --package hestia-daemon --bin hestia-daemon \
  --package hestiactl --bin hestiactl --locked --offline
hestia="$workspace/hestia/target/debug/hestia-daemon"
hestiactl="$workspace/hestia/target/debug/hestiactl"

python3 - "$scratch" "$store" "$llama_server" "$qwen_source" <<'PYTHON'
from pathlib import Path
import json, socket, sys

scratch, store = Path(sys.argv[1]), Path(sys.argv[2])
llama_server, qwen_source = sys.argv[3], sys.argv[4]

def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]

models = [
    {
        "artifact": "smollm3-3b-q4-k-m", "profile": "smollm3-theorist",
        "identity": "huggingfacetb/smollm3-3b",
        "revision": "4965cb60b150737b68a0408c36aeefb65078f894",
        "size": 1915305312,
        "digest": "8334b850b7bd46238c16b0c550df2138f0889bf433809008cc17a8b05761863e",
        "source": str(store / "8334b850b7bd46238c16b0c550df2138f0889bf433809008cc17a8b05761863e"),
        "memory": 4 * 1024 * 1024 * 1024,
    },
    {
        "artifact": "olmo2-1b-instruct-q4-k-m", "profile": "olmo-rigor-reviewer",
        "identity": "allenai/olmo-2-0425-1b-instruct",
        "revision": "62f8c199538474c3e33ed5d7e0580abd66686a27",
        "size": 935515296,
        "digest": "abd8187934a438fbf7cfff0a1de5b9d2793ce913f158794df1951dcba6c93cc6",
        "source": str(store / "abd8187934a438fbf7cfff0a1de5b9d2793ce913f158794df1951dcba6c93cc6"),
        "memory": 3 * 1024 * 1024 * 1024,
    },
    {
        "artifact": "granite4-h-tiny-q4-k-m", "profile": "granite-stakeholder",
        "identity": "ibm-granite/granite-4.0-h-tiny",
        "revision": "08d5a8a9741dd5c1a95d2d39e25253226aa1464e",
        "size": 4230976352,
        "digest": "5a38b08c441ae1adbafb1d2b8a7167e0d48734d83af68b268cefea1eec553dcd",
        "source": str(store / "5a38b08c441ae1adbafb1d2b8a7167e0d48734d83af68b268cefea1eec553dcd"),
        "memory": 8 * 1024 * 1024 * 1024,
    },
    {
        "artifact": "qwen25-coder-15b-instruct-q4-k-m", "profile": "qwen-code-historian",
        "identity": "qwen/qwen2.5-coder-1.5b-instruct",
        "revision": "2ab9f8f42af02fc212effaef7c4850c885e965f4",
        "size": 1117320768,
        "digest": "cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046",
        "source": qwen_source,
        "memory": 3 * 1024 * 1024 * 1024,
    },
]
profiles, entries = [], []
for index, item in enumerate(models, start=1):
    item["port"] = free_port()
    model = {
        "identity": item["identity"],
        "digest": list(bytes.fromhex(item["digest"])),
        "quantization": "Q4_K_M", "context_tokens": 4096,
    }
    arguments = [
        "--host", "127.0.0.1", "--port", str(item["port"]),
        "--ctx-size", "4096", "--parallel", "1",
        "--alias", item["identity"], "--jinja", "--no-webui", "--model",
    ]
    profiles.append({
        "id": item["profile"], "endpoint_reference": f"ensemble-provider-{index}",
        "executable": llama_server, "arguments": arguments,
        "artifact": {"artifact": item["artifact"], "argument_index": len(arguments)},
        "environment": [], "readiness_address": f"127.0.0.1:{item['port']}",
        "startup_timeout_ms": 120000, "probe_interval_ms": 100,
        "drain_timeout_ms": 15000,
    })
    entries.append({
        "entry": {
            "artifact": item["artifact"], "model": model, "registry": "huggingface",
            "source_revision": item["revision"], "license": "Apache-2.0",
            "format": "Gguf", "expected_size_bytes": item["size"],
            "expected_digest": list(bytes.fromhex(item["digest"])),
        },
        "executable": "/bin/cat", "arguments": [], "source": item["source"],
    })
    (scratch / f"acquire-{index}.json").write_text(json.dumps({
        "authority": [12] * 16, "artifact": item["artifact"],
        "operation": [30 + index] * 16, "expected_revision": 1,
        "accepted_license": "Apache-2.0",
        "expected_digest": list(bytes.fromhex(item["digest"])),
    }))
    (scratch / f"start-{index}.json").write_text(json.dumps({
        "authority": [12] * 16, "runtime": [20 + index] * 16,
        "operation": [40 + index] * 16, "expected_revision": 0,
        "expected_incarnation": 0, "hosting_profile": item["profile"],
        "model": model,
        "resources": {"memory_bytes": item["memory"], "accelerator": "metal", "threads": 8, "concurrency": 1},
        "sandbox": "Inherited", "egress": "Inherited",
    }))

(scratch / "hestia.json").write_text(json.dumps({
    "version": 1, "socket": str(scratch / "hestia.sock"),
    "authority": [12] * 16, "max_runtimes": 4, "profiles": profiles,
    "artifacts": {"directory": str(store), "max_inflight": 4, "entries": entries},
}))
(scratch / "models.json").write_text(json.dumps(models, indent=2))
PYTHON
chmod 600 "$scratch"/*.json

"$hestia" "$scratch/hestia.json" >"$scratch/hestia.out" 2>"$scratch/hestia.err" &
hestia_pid=$!
attempt=0
while [ ! -S "$scratch/hestia.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$hestia_pid" 2>/dev/null || [ "$attempt" -ge 400 ]; then
    sed -n '1,200p' "$scratch/hestia.err" >&2
    echo 'FAIL: Hestia ensemble owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

index=1
while [ "$index" -le 4 ]; do
  artifact=$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[$((index - 1))]['artifact'])")
  attempt=0
  while :; do
    observation=$("$hestiactl" artifact "$scratch/hestia.sock" "$artifact")
    lifecycle=$(printf '%s' "$observation" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["lifecycle"])')
    if [ "$lifecycle" = Admitted ]; then
      break
    fi
    if [ "$lifecycle" = Cataloged ]; then
      "$hestiactl" acquire "$scratch/hestia.sock" "$scratch/acquire-$index.json" >/dev/null
    elif [ "$lifecycle" = Failed ]; then
      printf '%s\n' "$observation" >&2
      echo "FAIL: Hestia could not admit $artifact" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    [ "$attempt" -lt 3600 ] || { echo "FAIL: admission timed out for $artifact" >&2; exit 1; }
    sleep 0.25
  done
  echo "Admitted ensemble artifact: $artifact"
  index=$((index + 1))
done

index=1
while [ "$index" -le 4 ]; do
  identity=$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[$((index - 1))]['identity'])")
  port=$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[$((index - 1))]['port'])")
  "$hestiactl" start "$scratch/hestia.sock" "$scratch/start-$index.json" >"$scratch/start-$index.out"
  python3 - "$scratch/start-$index.out" "$scratch/drain-$index.json" "$index" <<'PYTHON'
from pathlib import Path
import json, sys
start = json.loads(Path(sys.argv[1]).read_text())
ready = start["result"]["Applied"]
assert ready["lifecycle"] == "Ready", start
index = int(sys.argv[3])
Path(sys.argv[2]).write_text(json.dumps({
    "authority": [12] * 16, "runtime": [20 + index] * 16,
    "operation": [50 + index] * 16, "expected_revision": ready["revision"],
    "expected_incarnation": ready["incarnation"],
}))
PYTHON
  chmod 600 "$scratch/drain-$index.json"
  python3 - "$port" "$identity" <<'PYTHON'
from urllib.request import urlopen
from urllib.error import HTTPError, URLError
import json, sys, time
port, identity = int(sys.argv[1]), sys.argv[2]
deadline = time.monotonic() + 240
while True:
    try:
        with urlopen(f"http://127.0.0.1:{port}/v1/models", timeout=10) as response:
            if any(item["id"] == identity for item in json.load(response)["data"]):
                break
    except (HTTPError, URLError, TimeoutError):
        pass
    if time.monotonic() >= deadline:
        raise TimeoutError(f"provider readiness expired for {identity}")
    time.sleep(0.5)
PYTHON
  echo "Ready ensemble provider $index: $identity at 127.0.0.1:$port"
  index=$((index + 1))
done

question='Does Hestia retained-artifact restart recovery preserve its admission boundary without introducing unsafe trust or unusable operator friction? Identify concrete defects, missing evidence, and the smallest next action; do not approve rollout.'
make -C "$workspace/moira" qualify-methodical-workflow-live-ensemble \
  MOIRA_METHODICAL_LIVE_DENDRITE_PORT="$dendrite_port" \
  MOIRA_METHODICAL_LIVE_ENSEMBLE_CONTEXT_FILE="$context" \
  MOIRA_METHODICAL_LIVE_ENSEMBLE_CONTEXT_LINE_RANGES="$context_ranges" \
  MOIRA_METHODICAL_LIVE_ENSEMBLE_OUTPUT_FILE="$scratch/member-reports.json" \
  MOIRA_METHODICAL_LIVE_ENSEMBLE_QUESTION="$question" \
  MOIRA_METHODICAL_LIVE_MODEL_1="$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[0]['identity'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_ADDRESS_1="127.0.0.1:$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[0]['port'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_2="$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[1]['identity'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_ADDRESS_2="127.0.0.1:$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[1]['port'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_3="$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[2]['identity'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_ADDRESS_3="127.0.0.1:$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[2]['port'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_4="$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[3]['identity'])")" \
  MOIRA_METHODICAL_LIVE_MODEL_ADDRESS_4="127.0.0.1:$(python3 -c "import json; print(json.load(open('$scratch/models.json'))[3]['port'])")"

python3 - "$scratch/member-reports.json" <<'PYTHON'
from pathlib import Path
import json, sys
reports = json.loads(Path(sys.argv[1]).read_text())
assert len(reports) == 4
assert len({model for model, _ in reports}) == 4
for model, report in reports:
    parsed = json.loads(report)
    assert parsed["schema"] == "moira.member-response.v1"
    print(f"{model}: {parsed['claims'][0]['statement']}")
PYTHON

index=1
while [ "$index" -le 4 ]; do
  "$hestiactl" drain "$scratch/hestia.sock" "$scratch/drain-$index.json" >"$scratch/drain-$index.out"
  grep -q '"Stopped"' "$scratch/drain-$index.out"
  index=$((index + 1))
done
"$hestiactl" shutdown "$scratch/hestia.sock" 0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c >/dev/null
wait "$hestia_pid"
hestia_pid=

echo 'PASS: Moira retained four independently attributed reports from four Dendrite-grounded Hestia providers and all runtimes drained cleanly'
