#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /tmp/workbench-chronos-mutation.XXXXXX)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/chronos/Cargo.toml" \
  --package chronos-standalone --bin chronos-service --locked --offline
cargo build --manifest-path "$repo/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

python3 - "$scratch" "$repo/target/debug/pantheon" \
  "$workspace/chronos/target/debug/chronos-service" <<'PYTHON'
from pathlib import Path
import json
import shlex
import socket
import subprocess
import sys
import time

scratch, workbench, service = sys.argv[1:]
state = Path(scratch) / "state"
state.mkdir(mode=0o700)
tmux_socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([
    workbench,
    "interactive",
    str(state),
    "--host-chronos",
    "--chronos-allow-writes",
    "--chronos-scheduling",
    "--chronos-cancel-receipts",
])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=5,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "chronos").stdout

def expect(marker, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if marker in visible:
            return
        time.sleep(0.025)
    raise AssertionError(f"Missing {marker!r} in hosted Chronos pane:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "chronos", *values)

def text(value):
    tmux("send-keys", "-t", "chronos", "-l", value)

def palette(query, expected):
    keys("C-p")
    text(query)
    expect(expected)
    keys("Enter")

try:
    tmux("new-session", "-d", "-x", "160", "-y", "40", "-s", "chronos", command)
    expect("Observed 0 tasks", timeout=30)

    keys("n")
    expect("Task batch")
    text("workbench-proof")
    keys("Tab")
    text("workbench-secondary")
    keys("Tab")
    text("workbench-tertiary")
    keys("Enter")
    expect("Applied.")

    keys("s")
    expect("Observed scheduling and")
    keys("e")
    expect("Enable scheduling controls?")
    keys("y")
    expect("Scheduling control applied.")

    keys("r")
    expect("Observed 3 tasks")
    keys("s")
    time.sleep(0.1)
    keys("b")
    expect("Concurrency capacity:")
    text("3")
    keys("Enter")
    expect("Set concurrency capacity to 3?")
    keys("y")
    expect("Scheduling control applied.")

    keys("r")
    expect("Observed 3 tasks")
    keys("s")
    time.sleep(0.1)
    text(" ")
    expect("1 task marked")
    keys("Down")
    time.sleep(0.1)
    text(" ")
    expect("2 tasks marked")
    keys("Down")
    time.sleep(0.1)
    text(" ")
    expect("3 tasks marked")
    keys("s")
    expect("3 marked task eligibilities")
    keys("d")
    expect("Not-before: Unix seconds or clear")
    text("100000")
    keys("Enter")
    expect("Set not-before to 100000.000000000 for 3 exact tasks")
    keys("y")
    expect("Applied 3/3 independently settled task eligibility controls.")

    keys("r")
    expect("Observed 3 tasks")
    keys("s")
    expect("3 marked task eligibilities")
    keys("t")
    expect("Set task pause to true for 3 exact tasks")
    keys("y")
    expect("Applied 3/3 independently settled task eligibility controls.")

    keys("r")
    expect("Observed 3 tasks")
    keys("s")
    time.sleep(0.1)
    keys("p")
    expect("Set global pause to true?")
    keys("y")
    expect("Scheduling control applied.")

    keys("r")
    expect("Observed 3 tasks")
    keys("c")
    expect("Cancel 3 marked tasks?")
    keys("y")
    expect("Applied 3/3 independently receipted cancellations")
    expect("applied=3 refused=0 unknown=0 not-sent=0")

    palette("inspect tui instrumentation", "Inspect TUI instrumentation")
    expect("chronos.schedule.eligibility-batch")
    expect("chronos.task.cancel-batch")
    keys("C-g")

    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "chronos", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not drain hosted Chronos")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)

for endpoint in (state / "chronos.sock", state / "chronos.control.sock"):
    if endpoint.exists():
        raise AssertionError(f"hosted shutdown retained {endpoint}")

owner = subprocess.Popen(
    [service, str(state), "--atomic-state"],
    stdout=subprocess.DEVNULL,
    stderr=subprocess.PIPE,
)
try:
    if owner.stderr.readline().strip() != b"Chronos local service ready":
        raise AssertionError("Chronos reopen did not become ready")

    def readback(version, command):
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(3)
            connection.connect(str(state / "chronos.sock"))
            connection.sendall(json.dumps({"version": version, "command": command}).encode() + b"\n")
            with connection.makefile("rb") as response:
                frame = response.readline(262145)
        if len(frame) > 262144 or not frame.endswith(b"\n"):
            raise AssertionError("invalid Chronos readback frame")
        reply = json.loads(frame)
        if reply["version"] != version:
            raise AssertionError("unexpected Chronos readback version")
        return reply["outcome"]

    task = readback(1, {"operation": "snapshot", "task": "workbench-proof"})
    secondary = readback(1, {"operation": "snapshot", "task": "workbench-secondary"})
    scheduling = readback(5, {
        "operation": "overview",
        "now": {"before_epoch": False, "seconds": "0", "nanos": 0},
    })
    if task["value"]["snapshot"]["phase"] != "cancelled":
        raise AssertionError("reopened task was not cancelled")
    tertiary = readback(1, {"operation": "snapshot", "task": "workbench-tertiary"})
    if secondary["value"]["snapshot"]["phase"] != "cancelled":
        raise AssertionError("reopened second task was not cancelled")
    if tertiary["value"]["snapshot"]["phase"] != "cancelled":
        raise AssertionError("reopened third task was not cancelled")
    if not scheduling["controls"]["paused"]:
        raise AssertionError("reopened scheduling was not paused")
    if scheduling["concurrency"]["capacity"] != 3:
        raise AssertionError("reopened capacity did not match")
    eligibility = readback(5, {
        "operation": "eligibility",
        "task": task["value"]["reference"],
    })["eligibility"]
    expected = {
        "paused": True,
        "not_before": {"before_epoch": False, "seconds": "100000", "nanos": 0},
    }
    if eligibility != expected:
        raise AssertionError(f"reopened eligibility did not match: {eligibility!r}")
    for sibling in (secondary, tertiary):
        sibling_eligibility = readback(5, {
            "operation": "eligibility",
            "task": sibling["value"]["reference"],
        })["eligibility"]
        if sibling_eligibility != expected:
            raise AssertionError(
                f"reopened sibling eligibility did not match: {sibling_eligibility!r}"
            )
finally:
    subprocess.run([service, "--shutdown", str(state)], check=True, timeout=5)
    owner.wait(timeout=5)
    owner.stderr.close()
if owner.returncode != 0:
    raise AssertionError(f"Chronos reopen exited with {owner.returncode}")
PYTHON

echo 'PASS: hosted Chronos -> Pantheon N-task eligibility and independently receipted cancellation -> clean shutdown -> exact durable reopen'
