#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-chronos-blocked-agenda.XXXXXX)
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
state = Path(scratch) / "state"
state.mkdir(mode=0o700)
tmux_socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([
    pantheon,
    "interactive",
    str(state),
    "--host-chronos",
    "--chronos-allow-writes",
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
    return tmux("capture-pane", "-p", "-t", "agenda").stdout

def expect(marker, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if marker in visible:
            return
        time.sleep(0.025)
    raise AssertionError(f"Missing {marker!r} in Pantheon pane:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "agenda", *values)

def text(value):
    tmux("send-keys", "-t", "agenda", "-l", value)

try:
    tmux("new-session", "-d", "-x", "180", "-y", "44", "-s", "agenda", command)
    keys("M-1")
    expect("Observed 0 tasks", timeout=30)

    keys("n")
    expect("Task batch")
    text("alignment-prerequisite")
    keys("Tab")
    text("product-manager-control")
    keys("Enter")
    expect("Review 2 exact task(s)")
    keys("Down", "p")
    expect("Priority: signed integer")
    text("10")
    keys("Enter", "d")
    expect("Dependencies: comma-separated exact task IDs")
    text("alignment-prerequisite")
    keys("Enter", "Enter")
    expect("Task graph submission applied.")

    keys("C-p")
    text("operator agenda")
    expect("Go to operator agenda")
    keys("Enter")
    expect("Blocked Chronos work · product-manager-control · P10")
    expect("inspect only")
    expect("1 owner-published prerequisite(s) remain incomplete")
    expect("Dependencies; follow an incomplete prerequisite")

    observed = subprocess.run(
        [pantheon, "headless", str(state), "--operation-id", "blocked-agenda-live"],
        capture_output=True,
        text=True,
        check=True,
        timeout=10,
    )
    payload = json.loads(observed.stdout)
    if not payload["complete"]:
        raise AssertionError("headless attachment was incomplete")
    matching = [
        entry for entry in payload["operator_agenda"]["entries"]
        if entry["resource_identity"].startswith("product-manager-control#")
    ]
    if len(matching) != 1:
        raise AssertionError(f"missing exact blocked headless entry: {matching!r}")
    entry = matching[0]
    if entry["kind"] != "dependency_blocked":
        raise AssertionError(f"unexpected agenda kind: {entry!r}")
    if entry["actionability"] != "inspect_only":
        raise AssertionError(f"blocked task became actionable: {entry!r}")
    if "1 owner-published prerequisite" not in entry["blocker"]:
        raise AssertionError(f"missing blocker orientation: {entry!r}")

    keys("C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "agenda", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not drain hosted Chronos")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

echo 'PASS: blocked Chronos work -> Pantheon TUI/headless operator-agenda parity -> clean drain'
