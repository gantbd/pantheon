#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pantheon-hestia-observe.XXXXXX")
hestia_pid=

cleanup() {
  if [ -n "$hestia_pid" ] && kill -0 "$hestia_pid" 2>/dev/null; then
    kill "$hestia_pid" 2>/dev/null || true
    wait "$hestia_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/artifacts"
chmod 700 "$scratch/artifacts"

cargo build --quiet --manifest-path "$workspace/hestia/Cargo.toml" \
  --package hestia-daemon --bin hestia-daemon \
  --package hestiactl --bin hestiactl --locked --offline
cargo build --quiet --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

hestia="$workspace/hestia/target/debug/hestia-daemon"
hestiactl="$workspace/hestia/target/debug/hestiactl"
pantheon="$root/target/debug/pantheon"
python=$(command -v python3)

python3 - "$scratch" "$python" <<'PY'
from hashlib import sha256
from pathlib import Path
import json, socket, sys

scratch, python = Path(sys.argv[1]), sys.argv[2]
payload = b"bounded Hestia observation fixture"
source = scratch / "fixture.gguf"
source.write_bytes(payload)
digest = list(sha256(payload).digest())
reservation = socket.socket()
reservation.bind(("127.0.0.1", 0))
port = reservation.getsockname()[1]
reservation.close()
model = {
    "identity": "fixture/model",
    "digest": None,
    "quantization": "fixture-q4",
    "context_tokens": 4096,
}
code = (
    "import socket,time; "
    "s=socket.socket(); s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); "
    f"s.bind(('127.0.0.1',{port})); s.listen(); time.sleep(60)"
)
owner = {
    "version": 1,
    "socket": str(scratch / "hestia.sock"),
    "authority": [1] * 16,
    "max_runtimes": 1,
    "profiles": [{
        "id": "bounded-fixture",
        "endpoint_reference": "fixture-loopback",
        "executable": python,
        "arguments": ["-c", code],
        "environment": [],
        "readiness_address": f"127.0.0.1:{port}",
        "startup_timeout_ms": 5000,
        "probe_interval_ms": 10,
        "drain_timeout_ms": 1000,
    }],
    "artifacts": {
        "directory": str(scratch / "artifacts"),
        "max_inflight": 1,
        "entries": [{
            "entry": {
                "artifact": "fixture-q4",
                "model": {
                    "identity": "fixture/model",
                    "digest": digest,
                    "quantization": "fixture-q4",
                    "context_tokens": 4096,
                },
                "registry": "fixture-registry",
                "source_revision": "revision-1",
                "license": "Apache-2.0",
                "format": "Gguf",
                "expected_size_bytes": len(payload),
                "expected_digest": digest,
            },
            "executable": "/bin/cat",
            "arguments": [],
            "source": str(source),
        }],
    },
}
start = {
    "authority": [1] * 16,
    "runtime": [2] * 16,
    "operation": [3] * 16,
    "expected_revision": 0,
    "expected_incarnation": 0,
    "hosting_profile": "bounded-fixture",
    "model": model,
    "resources": {
        "memory_bytes": 1024,
        "accelerator": "cpu",
        "threads": 1,
        "concurrency": 1,
    },
    "sandbox": "Inherited",
    "egress": "Inherited",
}
for name, document in (("hestia.json", owner), ("start.json", start)):
    (scratch / name).write_text(json.dumps(document))
PY
chmod 600 "$scratch/hestia.json" "$scratch/start.json"

"$hestia" "$scratch/hestia.json" >"$scratch/hestia.out" 2>"$scratch/hestia.err" &
hestia_pid=$!
attempt=0
while [ ! -S "$scratch/hestia.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$hestia_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/hestia.err" >&2
    exit 1
  fi
  sleep 0.025
done

"$pantheon" hestia-artifact-catalog "$scratch/hestia.sock" >"$scratch/catalog.json"
"$pantheon" hestia-artifact-observe "$scratch/hestia.sock" fixture-q4 \
  >"$scratch/artifact.json"
if "$pantheon" hestia-artifact-observe "$scratch/hestia.sock" missing-artifact \
  >"$scratch/missing.json" 2>"$scratch/missing.err"; then
  echo "missing Hestia artifact was accepted" >&2
  exit 1
fi

"$hestiactl" start "$scratch/hestia.sock" "$scratch/start.json" >"$scratch/start.out"
"$pantheon" hestia-runtime-observe "$scratch/hestia.sock" \
  02020202020202020202020202020202 >"$scratch/runtime.json"

python3 - "$scratch" <<'PY'
from pathlib import Path
import json, sys

scratch = Path(sys.argv[1])
catalog_raw = (scratch / "catalog.json").read_text()
catalog = json.loads(catalog_raw)
assert catalog["capability"] == "artifacts.catalog.v1"
assert catalog["complete"] is True
assert catalog["evidence"]["count"] == 1
assert catalog["evidence"]["catalog"][0]["artifact"] == "fixture-q4"

artifact_raw = (scratch / "artifact.json").read_text()
artifact = json.loads(artifact_raw)
assert artifact["capability"] == "artifacts.observe.v1"
assert artifact["evidence"]["observation"]["lifecycle"] == "cataloged"
assert artifact["evidence"]["observation"]["entry"]["license"] == "Apache-2.0"

missing = json.loads((scratch / "missing.json").read_text())
assert missing["complete"] is False
assert missing["status"] == "not_found"

runtime_raw = (scratch / "runtime.json").read_text()
runtime = json.loads(runtime_raw)
observation = runtime["evidence"]["observation"]
assert runtime["capability"] == "runtimes.observe.v1"
assert runtime["complete"] is True
assert observation["runtime"] == "02" * 16
assert observation["lifecycle"] == "ready"
assert observation["readiness"]["disposition"] == "ready"
assert observation["readiness"]["basis"] == "tcp_accept"
assert observation["readiness"]["endpoint"]["scope"] == "loopback"

for label, raw in (("catalog", catalog_raw), ("artifact", artifact_raw), ("runtime", runtime_raw)):
    assert str(scratch) not in raw, f"{label} leaked scratch path: {raw}"
    assert '"executable"' not in raw
    assert '"arguments"' not in raw
    assert '"environment"' not in raw
    assert '"storage_path"' not in raw
    assert '"source"' not in raw
    assert '"registry_locator"' not in raw
    assert "python" not in raw.lower()

drain = {
    "authority": [1] * 16,
    "runtime": [2] * 16,
    "operation": [4] * 16,
    "expected_revision": observation["revision"],
    "expected_incarnation": observation["incarnation"],
}
(scratch / "drain.json").write_text(json.dumps(drain))
PY
chmod 600 "$scratch/drain.json"

"$hestiactl" drain "$scratch/hestia.sock" "$scratch/drain.json" >"$scratch/drain.out"
"$hestiactl" shutdown "$scratch/hestia.sock" 01010101010101010101010101010101 \
  >"$scratch/shutdown.out"
wait "$hestia_pid"
hestia_pid=
