#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pantheon-reconfigure-hosted.XXXXXX")

cleanup() {
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted Atlas",' \
  '  "surfaces": ["overview", "operations"],' \
  '  "agora": null,' \
  '  "moira": null,' \
  '  "daimon": null,' \
  '  "hermes": null,' \
  '  "aegis": null,' \
  '  "tellus": null,' \
  '  "chronos": {' \
  '    "deployment": "hosted",' \
  '    "workstreams": [{"name": "Atlas", "directory": "chronos-a"}],' \
  '    "requires": ["tasks.observe.v1"],' \
  '    "moira": [],' \
  '    "daimon": [],' \
  '    "planning": null' \
  '  },' \
  '  "vellum": null' \
  '}' >"$scratch/current.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted Borealis",' \
  '  "surfaces": ["overview", "operations"],' \
  '  "agora": null,' \
  '  "moira": null,' \
  '  "daimon": null,' \
  '  "hermes": null,' \
  '  "aegis": null,' \
  '  "tellus": null,' \
  '  "chronos": {' \
  '    "deployment": "hosted",' \
  '    "workstreams": [{"name": "Borealis", "directory": "chronos-b"}],' \
  '    "requires": ["tasks.observe.v1"],' \
  '    "moira": [],' \
  '    "daimon": [],' \
  '    "planning": null' \
  '  },' \
  '  "vellum": null' \
  '}' >"$scratch/next.json"

python3 - "$scratch" "$root/target/debug/pantheon" <<'PYTHON'
from pathlib import Path
import json
import shlex
import subprocess
import sys
import time

scratch, binary = sys.argv[1:]
scratch = Path(scratch)
tmux_socket = str(scratch / "tmux.sock")
command = shlex.join([
    binary,
    "interactive",
    "--workspace-profile",
    str(scratch / "current.json"),
]) + " 2>" + shlex.quote(str(scratch / "pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=5,
    )

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        captured = tmux("capture-pane", "-p", "-t", "hosted-reconfigure", check=False)
        if captured.returncode != 0:
            errors = (scratch / "pantheon.err").read_text() if (scratch / "pantheon.err").exists() else ""
            raise AssertionError(f"Pantheon session exited before {markers!r}:\n{errors}")
        screen = captured.stdout
        if all(marker in screen for marker in markers):
            return screen
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "150", "-y", "30", "-s", "hosted-reconfigure", command)
    expect("health: 1 current", "chronos / tasks: current")
    tmux("send-keys", "-t", "hosted-reconfigure", "C-p")
    tmux("send-keys", "-t", "hosted-reconfigure", "-l", "preflight and restart workspace profile")
    expect("Preflight and restart workspace profile")
    tmux("send-keys", "-t", "hosted-reconfigure", "Enter")
    tmux("send-keys", "-t", "hosted-reconfigure", "-l", str(scratch / "next.json"))
    tmux("send-keys", "-t", "hosted-reconfigure", "Enter")
    expect(
        "minimum action       full_local_topology_restart",
        "chronos · changed · full_local_topology_restart_required",
    )
    tmux("send-keys", "-t", "hosted-reconfigure", "c", "Enter")
    expect("health: 1 current", "chronos / tasks: current", timeout=15)

    old = subprocess.run(
        [binary, "headless", "--workspace-profile", str(scratch / "current.json")],
        capture_output=True,
        text=True,
        check=True,
        timeout=15,
    )
    report = json.loads(old.stdout)
    assert report["complete"]
    assert report["hosted_shutdown"]["status"] == "clean"
    assert report["hosted_shutdown"]["receipts"][0]["peer"] == "chronos"
    assert report["hosted_shutdown"]["receipts"][0]["disposition"] == "clean"

    tmux("send-keys", "-t", "hosted-reconfigure", "]")
    expect("topology: local_host_ready", "• chronos/Borealis", "reconfiguration: allowed")
    tmux("send-keys", "-t", "hosted-reconfigure", "C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "hosted-reconfigure", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            screen = tmux("capture-pane", "-p", "-t", "hosted-reconfigure").stdout
            raise AssertionError(f"hosted replacement did not drain on exit:\n{screen}")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

echo 'PASS: hosted Chronos profile diff -> clean old drain -> process replacement -> new hosted topology -> independently reopened old topology'
