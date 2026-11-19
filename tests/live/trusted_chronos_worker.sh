#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-trusted-worker.XXXXXX)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$repo/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

python3 - "$scratch" "$repo/target/debug/pantheon" <<'PYTHON'
from pathlib import Path
import json
import shlex
import subprocess
import sys
import time

scratch, pantheon = sys.argv[1:]
scratch = Path(scratch)
profile = scratch / "workspace.json"
profile.write_text(json.dumps({
    "version": 1,
    "name": "Trusted worker qualification",
    "surfaces": ["operations"],
    "chronos": {
        "deployment": "hosted",
        "workstreams": [{"name": "Product", "directory": "chronos"}],
        "requires": ["tasks.observe.v1"],
        "moira": [],
        "daimon": [],
        "trusted_worker": {
            "workstream": "Product",
            "agent": "product-manager",
            "grant_validity_seconds": 60,
            "lease_duration_seconds": 30,
        },
        "planning": None,
    },
}))

management = subprocess.run(
    [pantheon, "manage", "--workspace-profile", str(profile),
     "--chronos-allow-writes", "--operation-id", "trusted-worker-live"],
    input="status\ndrain\n", text=True, capture_output=True, check=True,
    timeout=20,
)
events = [json.loads(line) for line in management.stdout.splitlines()]
assert len(events) == 3, events
for event in events[:2]:
    worker = event["topology"]["trusted_chronos_worker"]
    assert worker["workstream"] == "Product", worker
    assert worker["state"] == "idle", worker
    assert "no grant, lease, task key, or result" in worker["authority_boundary"], worker
assert events[2]["event"] == "drain" and events[2]["complete"] is True, events[2]

tmux_socket = str(scratch / "tmux.sock")
command = " ".join([
    shlex.join([pantheon, "interactive", "--workspace-profile", str(profile),
                "--chronos-allow-writes"]),
    "; run_status=$?; printf '\n[trusted-worker-exit:%s]\n' \"$run_status\"; sleep 10",
])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=10,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "trusted-worker").stdout

def expect(*markers, timeout=30):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r}:\n{visible}")

try:
    tmux("new-session", "-d", "-x", "160", "-y", "36", "-s", "trusted-worker", command)
    expect("OPERATIONS", "LOCAL TOPOLOGY", "trusted worker idle", "private capability")
    tmux("send-keys", "-t", "trusted-worker", "C-q")
    expect("[trusted-worker-exit:0]", timeout=10)
finally:
    tmux("kill-server", check=False)

print("PASS: trusted Chronos worker is retained privately and visible in management/TUI status")
PYTHON
