#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/tmp/workbench-vellum-atomic.XXXXXX")
tmux_socket="$scratch/tmux.sock"
credential="$scratch/vellum.credential"

cleanup() {
  tmux -S "$tmux_socket" kill-server >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

printf 'alpha alpha one' >"$scratch/one.txt"
printf 'alpha two' >"$scratch/two.txt"
printf 'three alpha' >"$scratch/three.txt"

cargo build --manifest-path "$workspace/vellum/Cargo.toml" \
  --package vellum-daemon --bin vellum-daemon --locked --offline \
  --target-dir "$root/target"
"$root/target/debug/vellum-daemon" --provision-credential "$credential"

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Vellum owner-atomic multi-document qualification",
  "surfaces": ["workspace"],
  "vellum": {
    "deployment": "hosted",
    "socket": "$scratch/vellum.sock",
    "credential": "$credential",
    "roots": [
      "$scratch/one.txt",
      "$scratch/two.txt",
      "$scratch/three.txt"
    ],
    "requires": [
      "workspace.inventory.v1",
      "workspace.edit.v1",
      "workspace.durable-save.execute.v1",
      "workspace.durable-save.lookup.v1"
    ]
  }
}
EOF

cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline
workbench="$root/target/debug/pantheon"

python3 - "$scratch" "$tmux_socket" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, tmux_socket, binary = sys.argv[1:]
error_log = str(Path(scratch) / "pantheon.err")
command = shlex.join([
    binary, "interactive", "--workspace-profile", str(Path(scratch) / "profile.json"),
    "--vellum-allow-edit", "--vellum-allow-save",
])
command = f"{command} 2>{shlex.quote(error_log)} || {{ status=$?; echo PANTHEON_EXIT:$status; cat {shlex.quote(error_log)}; sleep 60; exit $status; }}"

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    captured = tmux("capture-pane", "-p", "-t", "vellum-atomic", check=False)
    if captured.returncode != 0:
        error = Path(error_log).read_text() if Path(error_log).exists() else "<no Pantheon stderr>"
        raise AssertionError(f"Pantheon tmux session ended before capture:\n{error}")
    return captured.stdout

def expect(*markers, timeout=15):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "vellum-atomic", *values)

def literal(value):
    tmux("send-keys", "-t", "vellum-atomic", "-l", value)

def palette(query, expected):
    keys("C-p")
    literal(query)
    expect(expected)
    keys("Enter")

try:
    tmux("new-session", "-d", "-x", "170", "-y", "32", "-s", "vellum-atomic", command)
    expect("alpha alpha one", "1/3", timeout=40)
    palette("toggle current vellum document selection", "Toggle current Vellum document selection")
    keys("F6")
    expect("alpha two", "2/3")
    palette("toggle current vellum document selection", "Toggle current Vellum document selection")
    keys("F6")
    expect("three alpha", "3/3")
    palette("toggle current vellum document selection", "Toggle current Vellum document selection")
    expect("3 Vellum document(s) selected for bounded work")

    palette(
        "replace text atomically across selected vellum documents",
        "Replace text atomically across selected Vellum documents",
    )
    expect("Vellum atomic replace", "One revision-bound in-memory transaction")
    literal("alpha")
    keys("Tab")
    literal("omega")
    keys("Enter")
    expect("4 matches across 3 exact documents", "document 1 @ revision 0")
    keys("c", "Enter")
    expect("published atomically across 3 changed documents")

    keys("q")
    expect("three omega", "Dirty")
    palette("inspect tui instrumentation", "Inspect TUI instrumentation")
    expect("vellum.atomic-replace", "owner_atomic_receipt", timeout=10)
    keys("C-g")

    palette("durably save selected vellum documents", "Durably save selected Vellum documents")
    expect("Vellum durable-save workset", "3 exact documents")
    keys("c", "Enter")
    expect("Vellum run settled: 3 durable", timeout=20)
    keys("C-q")
    deadline = time.monotonic() + 15
    while tmux("has-session", "-t", "vellum-atomic", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after atomic edit and durable settlement")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

test "$(cat "$scratch/one.txt")" = 'omega omega one'
test "$(cat "$scratch/two.txt")" = 'omega two'
test "$(cat "$scratch/three.txt")" = 'three omega'
test ! -e "$scratch/vellum.sock"
test ! -e "$scratch/vellum.sock.control"

echo 'PASS: marked set -> exact N-document preview -> one owner-atomic edit -> traced receipt -> independent durable saves'
