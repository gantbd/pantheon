#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pha.XXXXXX")
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

cargo build --manifest-path "$workspace/hestia/Cargo.toml" \
  --package hestia-daemon --bin hestia-daemon \
  --package hestiactl --bin hestiactl --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

hestia="$workspace/hestia/target/debug/hestia-daemon"
hestiactl="$workspace/hestia/target/debug/hestiactl"
workbench="$root/target/debug/pantheon"

python3 - "$scratch" <<'PYTHON'
from hashlib import sha256
from pathlib import Path
import json, socket, sys

scratch = Path(sys.argv[1])
payload = b"bounded daemon artifact"
source = scratch / "fixture.gguf"
source.write_bytes(payload)
digest = list(sha256(payload).digest())
reservation = socket.socket()
reservation.bind(("127.0.0.1", 0))
port = reservation.getsockname()[1]
reservation.close()
owner = {
    "version": 1,
    "socket": str(scratch / "hestia.sock"),
    "authority": [1] * 16,
    "max_runtimes": 1,
    "profiles": [{
        "id": "unused-fixture",
        "endpoint_reference": "unused-loopback",
        "executable": "/bin/sleep",
        "arguments": ["1"],
        "environment": [],
        "readiness_address": f"127.0.0.1:{port}",
        "startup_timeout_ms": 1000,
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
                    "quantization": "Q4_K_M",
                    "context_tokens": 4096,
                },
                "registry": "fixture",
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
(scratch / "hestia.json").write_text(json.dumps(owner))
(scratch / "profile.json").write_text(json.dumps({
    "version": 1,
    "name": "Hestia artifact qualification",
    "surfaces": ["hestia"],
    "hestia": {
        "socket": str(scratch / "hestia.sock"),
        "artifact_catalog": True,
    },
}))
PYTHON
chmod 600 "$scratch/hestia.json" "$scratch/profile.json"

"$hestia" "$scratch/hestia.json" >"$scratch/hestia.out" 2>"$scratch/hestia.err" &
hestia_pid=$!
attempt=0
while [ ! -S "$scratch/hestia.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$hestia_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/hestia.err" >&2
    echo 'FAIL: Hestia artifact owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
scratch = Path(scratch)
tmux_socket = str(scratch / "tmux.sock")
command = shlex.join([
    binary, "interactive", "--workspace-profile", str(scratch / "profile.json")
])
command += " 2>" + shlex.quote(str(scratch / "pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "artifact").stdout

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    current = ""
    while time.monotonic() < deadline:
        current = screen()
        if all(marker in current for marker in markers):
            return current
        time.sleep(0.025)
    error = (scratch / "pantheon.err").read_text(errors="replace")
    raise AssertionError(f"Missing {markers!r}:\n{current}\n{error}")

try:
    tmux("new-session", "-d", "-x", "190", "-y", "38", "-s", "artifact", command)
    tmux("send-keys", "-t", "artifact", "M-i")
    expect(
        "HESTIA · ACQUISITION",
        "fixture-q4",
        "source revision    revision-1",
        "license            Apache-2.0",
        "expected bytes     23",
        "lifecycle          Cataloged",
        "admission does not instantiate, qualify, invoke, or schedule",
        timeout=20,
    )
    tmux("send-keys", "-t", "artifact", "a")
    expect("REVIEW", "human gate", "armed              false")
    tmux("send-keys", "-t", "artifact", "c")
    expect("ARMED", "armed              true")
    tmux("send-keys", "-t", "artifact", "Enter")
    expect("ACCEPTED", timeout=20)
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        tmux("send-keys", "-t", "artifact", "r")
        time.sleep(0.05)
        if "lifecycle          Admitted" in screen():
            break
    expect("lifecycle          Admitted", "observed bytes     23")
    tmux("send-keys", "-t", "artifact", "C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "artifact", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit cleanly")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
hestia = next(peer for peer in report["peers"] if peer["peer"] == "hestia")
assert hestia["status"] == "current"
assert hestia["capabilities"]["artifacts.catalog.v1"] is True
assert hestia["capabilities"]["runtimes.observe.v1"] is False
artifact = hestia["artifacts"]["catalog"][0]
assert artifact["lifecycle"] == "Admitted"
assert artifact["observed_bytes"] == 23
assert "source" not in artifact["entry"]
assert "does not instantiate, qualify, invoke, or schedule" in hestia["artifacts"]["boundary"]
'

test "$(find "$scratch/artifacts" -type f | wc -l | tr -d ' ')" = 1
test -z "$(find "$root" -path "$root/target" -prune -o \( -name '*.gguf' -o -name '*.safetensors' \) -print)"
"$hestiactl" shutdown "$scratch/hestia.sock" 01010101010101010101010101010101 >/dev/null
wait "$hestia_pid"
hestia_pid=

echo 'PASS: Hestia artifact acquisition is human-gated, observable in TUI/headless, and private to the owner store'
