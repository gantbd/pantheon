#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/tmp/pantheon-vellum-rotation.XXXXXX")
tmux_socket="$scratch/tmux.sock"
credential="$scratch/vellum.credential"

cleanup() {
  tmux -S "$tmux_socket" kill-server >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
printf 'anchor' >"$scratch/source.txt"

cargo build --manifest-path "$workspace/vellum/Cargo.toml" \
  --package vellum-daemon --bin vellum-daemon --locked --offline \
  --target-dir "$root/target"
"$root/target/debug/vellum-daemon" --provision-credential "$credential"

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Vellum replay identity rotation qualification",
  "surfaces": ["workspace"],
  "vellum": {
    "deployment": "hosted",
    "socket": "$scratch/vellum.sock",
    "credential": "$credential",
    "roots": ["$scratch/source.txt"],
    "requires": ["workspace.inventory.v1", "workspace.edit.v1"]
  }
}
EOF

cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

python3 - "$scratch" "$tmux_socket" "$root/target/debug/pantheon" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, tmux_socket, binary = sys.argv[1:]
command = shlex.join([
    binary, "interactive", "--workspace-profile", str(Path(scratch) / "profile.json"),
    "--vellum-allow-edit", "--vellum-allow-save",
])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "vellum-rotation").stdout

def expect(*markers, timeout=15):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "vellum-rotation", *values)

def literal(value):
    tmux("send-keys", "-t", "vellum-rotation", "-l", value)

tmux("new-session", "-d", "-x", "140", "-y", "30", "-s", "vellum-rotation", command)
expect("anchor", "Service Current / Clean", timeout=40)
keys("Tab")
literal("x" * 190)
expect("Service Current / Dirty")
time.sleep(1.2)
literal("y" * 190)
time.sleep(1.2)
keys("C-s")
expect("Service Current / Clean", timeout=20)
keys("C-p")
literal("inspect tui instrumentation")
keys("Enter")
expect("vellum.client.renew-identity", timeout=10)
keys("C-g", "C-q")

deadline = time.monotonic() + 15
while tmux("has-session", "-t", "vellum-rotation", check=False).returncode == 0:
    if time.monotonic() >= deadline:
        raise AssertionError("Pantheon did not exit after the rotation journey")
    time.sleep(0.025)

actual = (Path(scratch) / "source.txt").read_text()
expected = "x" * 190 + "y" * 190 + "anchor"
if actual != expected:
    raise AssertionError(f"durable source mismatch: {len(actual)} != {len(expected)}")
PYTHON

printf '%s\n' 'PASS: two bounded client identities preserve one editor projection and durable source'
