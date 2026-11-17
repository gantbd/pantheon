#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/tmp/workbench-vellum-batch.XXXXXX")
tmux_socket="$scratch/tmux.sock"
credential="$scratch/vellum.credential"

cleanup() {
  tmux -S "$tmux_socket" kill-server >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

printf 'alpha' >"$scratch/one.txt"
printf 'bravo' >"$scratch/two.txt"
printf 'charlie' >"$scratch/three.txt"

cargo build --manifest-path "$workspace/vellum/Cargo.toml" \
  --package vellum-daemon --bin vellum-daemon --locked --offline \
  --target-dir "$root/target"
"$root/target/debug/vellum-daemon" --provision-credential "$credential"

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Vellum bounded durable-save qualification",
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
command = shlex.join([
    binary, "interactive", "--workspace-profile", str(Path(scratch) / "profile.json"),
    "--vellum-allow-edit", "--vellum-allow-save",
])
error_log = str(Path(scratch) / "pantheon.err")
command = f"{command} 2>{shlex.quote(error_log)} || {{ status=$?; echo PANTHEON_EXIT:$status; cat {shlex.quote(error_log)}; sleep 60; exit $status; }}"

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    captured = tmux("capture-pane", "-p", "-t", "vellum-batch", check=False)
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
    tmux("send-keys", "-t", "vellum-batch", *values)

def palette(query, expected):
    keys("C-p")
    tmux("send-keys", "-t", "vellum-batch", "-l", query)
    expect(expected)
    keys("Enter")

try:
    tmux("new-session", "-d", "-x", "170", "-y", "30", "-s", "vellum-batch", command)
    expect("alpha", "1/3", timeout=40)
    keys("Tab", "X")
    expect("Xalpha", "Dirty")
    palette("toggle current vellum document selection", "Toggle current Vellum document selection")
    expect("1 Vellum document(s) selected for bounded work")

    keys("F6", "Y")
    expect("Ybravo", "2/3")
    palette("toggle current vellum document selection", "Toggle current Vellum document selection")
    expect("2 Vellum document(s) selected for bounded work")

    keys("F6", "Z")
    expect("Zcharlie", "3/3")
    palette("toggle current vellum document selection", "Toggle current Vellum document selection")
    expect("3 Vellum document(s) selected for bounded work")
    palette("durably save selected vellum documents", "Durably save selected Vellum documents")
    expect("Vellum durable-save workset", "3 exact documents")
    keys("c", "Enter")
    visible = expect("Vellum run settled: 3 durable", timeout=20)
    if visible.count("Durable") < 3:
        raise AssertionError(f"Per-document durable receipts were not visible:\n{visible}")
    keys("C-q")
    deadline = time.monotonic() + 15
    while tmux("has-session", "-t", "vellum-batch", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit after durable Vellum settlement")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON

test "$(cat "$scratch/one.txt")" = Xalpha
test "$(cat "$scratch/two.txt")" = Ybravo
test "$(cat "$scratch/three.txt")" = Zcharlie
test ! -e "$scratch/vellum.sock"
test ! -e "$scratch/vellum.sock.control"

echo 'PASS: three Vellum documents -> retained dirty workset -> independent durable receipts -> exact persisted bytes'
