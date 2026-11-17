#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-make-run.XXXXXX)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"
printf 'make-run proof\n' >"$scratch/source.txt"

python3 - "$scratch" "$repo" <<'PYTHON'
from pathlib import Path
import subprocess
import sys
import time

scratch, repo = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
command = (
    f"make run PANTHEON_RUN_ROOT={scratch} "
    f"PANTHEON_RUN_VELLUM_ROOT={Path(scratch) / 'source.txt'}; "
    "run_status=$?; printf '\\n[make-run-exit:%s]\\n' \"$run_status\"; sleep 30; exit \"$run_status\""
)

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=10,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "make-run").stdout

def expect(*markers, timeout=40):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in make run:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "make-run", *values)

try:
    tmux(
        "new-session",
        "-d",
        "-x",
        "200",
        "-y",
        "42",
        "-c",
        repo,
        "-s",
        "make-run",
    )
    tmux("send-keys", "-t", "make-run", "-l", command)
    tmux("send-keys", "-t", "make-run", "Enter")
    expect("PANTHEON", "CURRENT", timeout=90)

    keys("M-1")
    expect("CHRONOS", "Observed 0 tasks")
    keys("n")
    expect("Task batch")
    keys("Escape")

    keys("M-2")
    expect("VELLUM")
    keys("C-o")
    expect("Open file (Enter/Esc):")
    keys("-l", str(Path(scratch) / "source.txt"))
    keys("Enter")
    expect("Vellum admitted and opened the file", "make-run proof")

    keys("C-q")
    expect("[make-run-exit:0]", timeout=10)
finally:
    tmux("kill-server", check=False)

print("PASS: make run provides writable Chronos and Ctrl-O-capable hosted Vellum")
PYTHON
