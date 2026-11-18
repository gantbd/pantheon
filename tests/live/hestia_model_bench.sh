#!/bin/sh
set -eu

if [ "${PANTHEON_LOCAL_MODEL_BENCH:-0}" != 1 ]; then
  echo 'SKIP: set PANTHEON_LOCAL_MODEL_BENCH=1 to acquire and exercise the specialist model bench'
  exit 0
fi

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=${PANTHEON_MODEL_BENCH_ROOT:-$(mktemp -d "/private/tmp/pmb.XXXXXX")}
source_root=${PANTHEON_MODEL_BENCH_SOURCE_ROOT:-}
hestia_pid=

cleanup() {
  if [ -n "$hestia_pid" ] && kill -0 "$hestia_pid" 2>/dev/null; then
    kill "$hestia_pid" 2>/dev/null || true
    wait "$hestia_pid" 2>/dev/null || true
  fi
  if [ "${PANTHEON_KEEP_MODEL_BENCH:-0}" = 1 ]; then
    echo "Retained specialist-model artifacts and evidence at $scratch"
  elif [ -z "${PANTHEON_MODEL_BENCH_ROOT:-}" ]; then
    rm -rf "$scratch"
  fi
}
trap cleanup EXIT HUP INT TERM

if [ -e "$scratch" ] && [ ! -d "$scratch" ]; then
  echo "FAIL: model-bench root is not a directory: $scratch" >&2
  exit 1
fi
if [ -n "$source_root" ] && [ ! -d "$source_root" ]; then
  echo "FAIL: model-bench source root is not a directory: $source_root" >&2
  exit 1
fi
mkdir -p "$scratch/artifacts" "$scratch/responses"
chmod 700 "$scratch" "$scratch/artifacts" "$scratch/responses"
if [ -S "$scratch/hestia.sock" ]; then
  rm "$scratch/hestia.sock"
fi

llama_server=${PANTHEON_LLAMA_SERVER:-/opt/homebrew/bin/llama-server}
if [ ! -x "$llama_server" ]; then
  echo "FAIL: llama-server is unavailable at $llama_server" >&2
  exit 1
fi

cargo build --manifest-path "$workspace/hestia/Cargo.toml" \
  --package hestia-daemon --bin hestia-daemon \
  --package hestiactl --bin hestiactl --locked --offline

hestia="$workspace/hestia/target/debug/hestia-daemon"
hestiactl="$workspace/hestia/target/debug/hestiactl"
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')

python3 - "$scratch" "$llama_server" "$port" "$source_root" <<'PYTHON'
from pathlib import Path
import json, sys

scratch, llama_server, port = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
source_root = Path(sys.argv[4]) if sys.argv[4] else None
models = [
    {
        "artifact": "smollm3-3b-q4-k-m",
        "profile": "smollm3-reasoning",
        "identity": "huggingfacetb/smollm3-3b",
        "revision": "4965cb60b150737b68a0408c36aeefb65078f894",
        "size": 1915305312,
        "digest": "8334b850b7bd46238c16b0c550df2138f0889bf433809008cc17a8b05761863e",
        "source": "https://huggingface.co/ggml-org/SmolLM3-3B-GGUF/resolve/4965cb60b150737b68a0408c36aeefb65078f894/SmolLM3-Q4_K_M.gguf",
        "memory": 4 * 1024 * 1024 * 1024,
        "prompt": "Act as a bounded delivery critic. List the three most important pieces of evidence needed before declaring a software task complete. Do not invent evidence.",
    },
    {
        "artifact": "olmo2-1b-instruct-q4-k-m",
        "profile": "olmo-verifier-small",
        "identity": "allenai/olmo-2-0425-1b-instruct",
        "revision": "62f8c199538474c3e33ed5d7e0580abd66686a27",
        "size": 935515296,
        "digest": "abd8187934a438fbf7cfff0a1de5b9d2793ce913f158794df1951dcba6c93cc6",
        "source": "https://huggingface.co/allenai/OLMo-2-0425-1B-Instruct-GGUF/resolve/62f8c199538474c3e33ed5d7e0580abd66686a27/OLMo-2-0425-1B-Instruct-Q4_K_M.gguf",
        "memory": 3 * 1024 * 1024 * 1024,
        "prompt": "Classify this test result as pass, fail, or uncertain and give one sentence of evidence: the process exited zero, but no expected output assertion was executed.",
    },
    {
        "artifact": "granite4-h-tiny-q4-k-m",
        "profile": "granite-business-sme",
        "identity": "ibm-granite/granite-4.0-h-tiny",
        "revision": "08d5a8a9741dd5c1a95d2d39e25253226aa1464e",
        "size": 4230976352,
        "digest": "5a38b08c441ae1adbafb1d2b8a7167e0d48734d83af68b268cefea1eec553dcd",
        "source": "https://huggingface.co/ibm-granite/granite-4.0-h-tiny-GGUF/resolve/08d5a8a9741dd5c1a95d2d39e25253226aa1464e/granite-4.0-h-tiny-Q4_K_M.gguf",
        "memory": 8 * 1024 * 1024 * 1024,
        "prompt": "Act as a product stakeholder. Given a proposal to let agents close work items automatically, state the user value, the primary risk, and the human decision gate. Stay within those facts.",
    },
]

profiles = []
entries = []
for item in models:
    model = {
        "identity": item["identity"],
        "digest": list(bytes.fromhex(item["digest"])),
        "quantization": "Q4_K_M",
        "context_tokens": 4096,
    }
    arguments = [
        "--host", "127.0.0.1", "--port", str(port),
        "--ctx-size", "4096", "--parallel", "1",
        "--alias", item["identity"], "--jinja", "--no-webui", "--model",
    ]
    profiles.append({
        "id": item["profile"],
        "endpoint_reference": f'{item["profile"]}-loopback',
        "executable": llama_server,
        "arguments": arguments,
        "artifact": {"artifact": item["artifact"], "argument_index": len(arguments)},
        "environment": [],
        "readiness_address": f"127.0.0.1:{port}",
        "startup_timeout_ms": 90000,
        "probe_interval_ms": 100,
        "drain_timeout_ms": 15000,
    })
    retained = source_root / item["digest"] if source_root is not None else None
    reuse = retained is not None and retained.is_file()
    entries.append({
        "entry": {
            "artifact": item["artifact"],
            "model": model,
            "registry": "huggingface",
            "source_revision": item["revision"],
            "license": "Apache-2.0",
            "format": "Gguf",
            "expected_size_bytes": item["size"],
            "expected_digest": list(bytes.fromhex(item["digest"])),
        },
        # A retained artifact is still copied through Hestia's bounded
        # acquisition worker and rehashed before admission. This avoids a
        # network fetch without treating an old pathname as trusted state.
        "executable": "/bin/cat" if reuse else "/usr/bin/curl",
        "arguments": [] if reuse else ["-fL", "--retry", "3", "--connect-timeout", "30", "--silent", "--show-error"],
        "source": str(retained) if reuse else item["source"],
    })

configuration = {
    "version": 1,
    "socket": str(scratch / "hestia.sock"),
    "authority": [12] * 16,
    "max_runtimes": len(models),
    "profiles": profiles,
    "artifacts": {
        "directory": str(scratch / "artifacts"),
        "max_inflight": 1,
        "entries": entries,
    },
}
(scratch / "hestia.json").write_text(json.dumps(configuration))
(scratch / "bench.json").write_text(json.dumps(models, indent=2))
for index, item in enumerate(models, start=1):
    digest = list(bytes.fromhex(item["digest"]))
    model = {
        "identity": item["identity"],
        "digest": digest,
        "quantization": "Q4_K_M",
        "context_tokens": 4096,
    }
    (scratch / f"acquire-{index}.json").write_text(json.dumps({
        "authority": [12] * 16,
        "artifact": item["artifact"],
        "operation": [30 + index] * 16,
        "expected_revision": 1,
        "accepted_license": "Apache-2.0",
        "expected_digest": digest,
    }))
    (scratch / f"start-{index}.json").write_text(json.dumps({
        "authority": [12] * 16,
        "runtime": [20 + index] * 16,
        "operation": [40 + index] * 16,
        "expected_revision": 0,
        "expected_incarnation": 0,
        "hosting_profile": item["profile"],
        "model": model,
        "resources": {
            "memory_bytes": item["memory"],
            "accelerator": "metal",
            "threads": 8,
            "concurrency": 1,
        },
        "sandbox": "Inherited",
        "egress": "Inherited",
    }))
PYTHON
chmod 600 "$scratch"/*.json

"$hestia" "$scratch/hestia.json" >"$scratch/hestia.out" 2>"$scratch/hestia.err" &
hestia_pid=$!
attempt=0
while [ ! -S "$scratch/hestia.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$hestia_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/hestia.err" >&2
    echo 'FAIL: Hestia model-bench owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

index=1
while [ "$index" -le 3 ]; do
  artifact=$(python3 -c "import json; print(json.load(open('$scratch/bench.json'))[$((index - 1))]['artifact'])")
  size=$(python3 -c "import json; print(json.load(open('$scratch/bench.json'))[$((index - 1))]['size'])")
  echo "Acquiring $artifact through Hestia ($size exact bytes)"
  "$hestiactl" acquire "$scratch/hestia.sock" "$scratch/acquire-$index.json" >/dev/null
  attempt=0
  while :; do
    observation=$("$hestiactl" artifact "$scratch/hestia.sock" "$artifact")
    lifecycle=$(printf '%s' "$observation" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["lifecycle"])')
    observed=$(printf '%s' "$observation" | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["observed_bytes"])')
    if [ "$lifecycle" = Admitted ]; then
      echo "  admitted $artifact after $observed verified bytes"
      break
    fi
    if [ "$lifecycle" = Failed ]; then
      printf '%s\n' "$observation" >&2
      echo "FAIL: Hestia acquisition failed for $artifact" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    if [ $((attempt % 20)) -eq 0 ]; then
      echo "  acquired $observed / $size bytes"
    fi
    if [ "$attempt" -ge 3600 ]; then
      echo "FAIL: Hestia acquisition timed out for $artifact" >&2
      exit 1
    fi
    sleep 0.5
  done
  index=$((index + 1))
done

index=1
while [ "$index" -le 3 ]; do
  identity=$(python3 -c "import json; print(json.load(open('$scratch/bench.json'))[$((index - 1))]['identity'])")
  prompt=$(python3 -c "import json; print(json.load(open('$scratch/bench.json'))[$((index - 1))]['prompt'])")
  echo "Starting $identity from its Hestia-admitted artifact"
  "$hestiactl" start "$scratch/hestia.sock" "$scratch/start-$index.json" >"$scratch/start-$index.out"
  python3 - "$scratch/start-$index.out" "$scratch/drain-$index.json" "$index" <<'PYTHON'
from pathlib import Path
import json, sys

start = json.loads(Path(sys.argv[1]).read_text())
ready = start["result"]["Applied"]
assert ready["lifecycle"] == "Ready", start
index = int(sys.argv[3])
Path(sys.argv[2]).write_text(json.dumps({
    "authority": [12] * 16,
    "runtime": [20 + index] * 16,
    "operation": [50 + index] * 16,
    "expected_revision": ready["revision"],
    "expected_incarnation": ready["incarnation"],
}))
PYTHON
  chmod 600 "$scratch/drain-$index.json"
  python3 - "$port" "$identity" "$prompt" "$scratch/responses/$index.json" <<'PYTHON'
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError
from pathlib import Path
import json, sys, time

port, identity, prompt, output = int(sys.argv[1]), sys.argv[2], sys.argv[3], Path(sys.argv[4])
base = f"http://127.0.0.1:{port}"
deadline = time.monotonic() + 240
while True:
    try:
        with urlopen(base + "/v1/models", timeout=10) as response:
            models = json.load(response)
        if any(item["id"] == identity for item in models["data"]):
            break
    except (HTTPError, URLError, TimeoutError):
        pass
    if time.monotonic() >= deadline:
        raise TimeoutError(f"provider readiness expired for {identity}")
    time.sleep(0.5)
request = Request(
    base + "/v1/chat/completions",
    data=json.dumps({
        "model": identity,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0,
        "max_tokens": 256,
        "chat_template_kwargs": {"enable_thinking": False},
    }).encode(),
    headers={"content-type": "application/json"},
)
with urlopen(request, timeout=180) as response:
    completion = json.load(response)
text = completion["choices"][0]["message"]["content"].strip()
assert text, completion
assert completion["usage"]["completion_tokens"] > 0, completion
output.write_text(json.dumps(completion, indent=2))
print("  response:", " ".join(text.split())[:240])
PYTHON
  "$hestiactl" drain "$scratch/hestia.sock" "$scratch/drain-$index.json" >"$scratch/drain-$index.out"
  grep -q '"Stopped"' "$scratch/drain-$index.out"
  index=$((index + 1))
done

"$hestiactl" shutdown "$scratch/hestia.sock" 0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c >/dev/null
wait "$hestia_pid"
hestia_pid=

echo 'PASS: Hestia acquired, verified, served, exercised, drained, and retained the three-model serving smoke bench'
