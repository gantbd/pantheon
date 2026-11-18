#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-emacs.XXXXXX)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/chronos"
chmod 700 "$scratch/chronos"
printf 'alpha\nbeta\n' >"$scratch/source.txt"

cargo build --manifest-path "$repo/Cargo.toml" -p pantheon-cli --bin pantheon \
  --locked --offline >/dev/null
binary="$repo/target/debug/pantheon"
profile="$scratch/workspace.json"
printf '%s\n' \
  '{"version":1,"name":"Emacs live gate","surfaces":["overview","chronos","vellum","configuration"],"keybindings":"emacs","chronos":{"deployment":"hosted","workstreams":[{"name":"Chronos","directory":"chronos"}],"requires":["tasks.observe.v1"],"moira":[],"daimon":[],"planning":null},"vellum":{"deployment":"hosted","socket":"vellum.sock","credential":null,"roots":["source.txt"],"allow_add_file":true,"requires":["workspace.inventory.v1"]}}' \
  >"$profile"
"$binary" interactive --workspace-profile "$profile" --check-profile \
  --vellum-allow-edit --vellum-allow-save >/dev/null

python3 - "$scratch" "$binary" "$profile" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
socket = str(Path(scratch) / "tmux.sock")
pantheon = shlex.join([
    binary,
    "interactive",
    "--workspace-profile",
    profile,
    "--vellum-allow-edit",
    "--vellum-allow-save",
])
command = (
    f"{pantheon}; status=$?; "
    "printf '\\n[pantheon-exit:%s]\\n' \"$status\"; sleep 30; exit \"$status\""
)

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=10,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "emacs").stdout

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r}:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "emacs", *values)

try:
    started = tmux(
        "new-session", "-d", "-x", "200", "-y", "42", "-s", "emacs", command,
        check=False,
    )
    if started.returncode != 0:
        raise AssertionError(f"Pantheon tmux session failed to start: {started.stderr}")
    expect("OVERVIEW", "M-x act", "C-x C-c exit", timeout=30)

    keys("C-x", "b")
    expect("MINIBUFFER", "M-x go to", "Go to Chronos", "Go to Vellum")
    keys("C-g")

    keys("C-x", "Right")
    expect("CHRONOS")
    keys("C-x", "Right")
    expect("VELLUM", "alpha", "beta")

    keys("C-s")
    expect("Find literal")
    keys("Escape")
    keys("C-o")
    expect("Command is unavailable in Vellum service mode")

    for key in ("C-a", "C-e", "C-b", "C-f", "C-n", "C-p", "C-d"):
        keys(key)
        expect("Vellum action accepted")

    keys("C-x", "C-f")
    expect("Open file (Enter/Esc):")
    keys("Escape")
    keys("C-x", "C-s")
    expect("Save submitted through Vellum's guarded service path", "Service Current / Clean")

    keys("C-x", "Left")
    expect("CHRONOS")
    keys("C-x", "C-c")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "emacs", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("C-x C-c did not close Pantheon")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)

print("PASS: Emacs profile uses real prefix chords, surface switching, Vellum motion/search, save and guarded exit")
PYTHON
