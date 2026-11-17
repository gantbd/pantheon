#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pantheon-reconfigure.XXXXXX")

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
  '  "name": "Before restart",' \
  '  "surfaces": ["overview"],' \
  '  "agora": null,' \
  '  "moira": null,' \
  '  "daimon": null,' \
  '  "hermes": null,' \
  '  "aegis": null,' \
  '  "grounding": null,' \
  '  "chronos": null,' \
  '  "vellum": null' \
  '}' >"$scratch/current.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "After restart",' \
  '  "surfaces": ["overview", "operations"],' \
  '  "agora": null,' \
  '  "moira": null,' \
  '  "daimon": null,' \
  '  "hermes": null,' \
  '  "aegis": null,' \
  '  "grounding": null,' \
  '  "chronos": null,' \
  '  "vellum": null' \
  '}' >"$scratch/next.json"

python3 - "$scratch" "$root/target/debug/pantheon" <<'PYTHON'
from pathlib import Path
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
])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=5,
    )

def expect(*markers, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "reconfigure").stdout
        if all(marker in screen for marker in markers):
            return screen
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "150", "-y", "30", "-s", "reconfigure", command)
    expect("PANTHEON", "Overview")
    tmux("send-keys", "-t", "reconfigure", "C-p")
    tmux("send-keys", "-t", "reconfigure", "-l", "preflight and restart workspace profile")
    expect("Preflight and restart workspace profile")
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect("Workspace profile restart", "type/paste path")
    tmux("send-keys", "-t", "reconfigure", "-l", str(scratch / "next.json"))
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect(
        "minimum action       shell_restart",
        "workspace.name · changed · shell_restart_required",
        "workspace.surfaces · changed · shell_restart_required",
    )
    tmux("send-keys", "-t", "reconfigure", "Escape")
    tmux("send-keys", "-t", "reconfigure", "C-p")
    tmux("send-keys", "-t", "reconfigure", "-l", "inspect tui instrumentation")
    expect("Inspect TUI instrumentation")
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect(
        "TUI instrumentation",
        "0 in flight · 0 budget violations",
        "workbench.reconfigure.preflight",
        "workbench.render",
    )
    tmux("send-keys", "-t", "reconfigure", "Escape")
    tmux("send-keys", "-t", "reconfigure", "C-p")
    tmux("send-keys", "-t", "reconfigure", "-l", "preflight and restart workspace profile")
    expect("Preflight and restart workspace profile")
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect("minimum action       shell_restart")
    next_profile = scratch / "next.json"
    next_profile.write_text(next_profile.read_text() + "\n")
    tmux("send-keys", "-t", "reconfigure", "c")
    expect("ARMED: Enter drains owned topology and restarts this process")
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect("Restart refused: profile bytes changed after preflight")
    tmux("send-keys", "-t", "reconfigure", "e", "Enter")
    expect("minimum action       shell_restart")
    tmux("send-keys", "-t", "reconfigure", "c")
    expect("ARMED: Enter drains owned topology and restarts this process")
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect("Overview  Operations", timeout=10)
    tmux("send-keys", "-t", "reconfigure", "C-p")
    tmux("send-keys", "-t", "reconfigure", "-l", "inspect session configuration")
    expect("Inspect session configuration")
    tmux("send-keys", "-t", "reconfigure", "Enter")
    expect("Session configuration", "surfaces            Overview · Operations")
    tmux("send-keys", "-t", "reconfigure", "Escape")
    time.sleep(0.1)
    tmux("send-keys", "-t", "reconfigure", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "reconfigure", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            screen = tmux("capture-pane", "-p", "-t", "reconfigure").stdout
            raise AssertionError(f"reconfigured Pantheon session did not exit:\n{screen}")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

echo 'PASS: validated profile diff -> explicit arm -> clean Ratatui process replacement -> replacement configuration evidence'
