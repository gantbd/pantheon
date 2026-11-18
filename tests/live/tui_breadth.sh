#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-breadth.XXXXXX)
cleanup() {
  tmux -S "$scratch/tmux.sock" kill-server >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

python3 - "$scratch" "$root/target/debug/pantheon" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary = sys.argv[1:]
scratch = Path(scratch)
tmux_socket = str(scratch / "tmux.sock")
chronos = str(scratch / "chronos")
command = shlex.join([
    binary,
    "interactive",
    chronos,
    "--host-chronos",
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
    return tmux("capture-pane", "-p", "-t", "breadth").stdout

def expect(*markers, timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        current = screen()
        if all(marker in current for marker in markers):
            return current
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{current}")

def send(*keys):
    tmux("send-keys", "-t", "breadth", *keys)

def send_literal(value):
    tmux("send-keys", "-t", "breadth", "-l", value)

try:
    tmux("new-session", "-d", "-x", "180", "-y", "36", "-s", "breadth", command)
    expect(
        "OVERVIEW",
        "Agenda",
        "Chronos",
        "Vellum",
        "Agora",
        "Moira",
        "Daimon",
        "Tellus",
        "Aegis",
        "Hermes",
        "Operations",
        "Configuration",
        timeout=8,
    )

    direct = [
        ("M-r", "AGENDA", "MBR / QBR READINESS"),
        ("M-c", "CHRONOS", "Observed 0 tasks"),
        ("M-v", "VELLUM", "Vellum service not configured"),
        ("M-a", "AGORA", "conversation workspace"),
        ("M-m", "MOIRA", "saga operations desk"),
        ("M-d", "DAIMON", "agent control room"),
        ("M-g", "TELLUS", "grounded methods and evidence"),
        ("M-e", "AEGIS", "authority console"),
        ("M-l", "HERMES", "ledger observatory"),
        ("M-o", "OPERATIONS", "LOCAL TOPOLOGY"),
        ("M-,", "CONFIGURATION", "LIVE SESSION FACTS"),
        ("M-h", "OVERVIEW", "LIVE SIGNALS"),
    ]
    for key, heading, evidence in direct:
        send(key)
        expect(heading, evidence)

    # Stable semantic direct keys do not depend on profile order. Terminals
    # that report Ctrl-number chords get the requested dashboard/component
    # map; Alt-number remains an equivalent portable encoding.
    send_literal("\x1b[48;5u")
    expect("OVERVIEW", "LIVE SIGNALS")
    send_literal("\x1b[57;5u")
    expect("OPERATIONS", "LOCAL TOPOLOGY")
    send("M-0")
    expect("OVERVIEW", "LIVE SIGNALS")

    # Inject one SGR left-click into the enabled mouse stream. At this width
    # column 14 is the Agenda title in the host-owned rail.
    send_literal("\x1b[<0;14;2M")
    expect(
        "AGENDA",
        "No current owner-published attention items",
        "no sovereign work-mode instance owner configured",
    )

    send("C-p")
    send_literal("go to operator agenda")
    send("Enter")
    expect("AGENDA", "agenda presence grants no authority")

    send_literal("\x1b[48;5u")
    expect("OVERVIEW", "LIVE SIGNALS")
    send("r")
    expect("Fresh observations queued across configured peers")

    send("C-p")
    expect("*Completions*", "MINIBUFFER", "M-x", "Go to overview")
    send_literal("show contextual help")
    send("Enter")
    expect("PANTHEON · FIELD GUIDE", "Ctrl-,", "Ctrl-P", "Ctrl-G", "Mouse")
    send("F1")
    send("F1")
    expect("PANTHEON · FIELD GUIDE", "Ctrl-,", "Ctrl-P", "Ctrl-G", "Mouse")
    send("F1")

    send("C-p")
    send_literal("inspect local provider configuration")
    send("Enter")
    expect("Local provider configuration", "No hosted Daimon owner configuration", "did not inspect a fallback file")
    send("r")
    expect("No hosted Daimon owner configuration", "Esc closes")
    send("C-g")

    send("C-p")
    send_literal("configure acp agent sessions")
    send("Enter")
    expect("ACP session configuration", "exact ACP/Daimon session", "agent_sessions")
    send("C-g")

    send("C-p")
    send_literal("configure agora room control")
    send("Enter")
    expect("Agora room configuration", "participant", "create an Agora room")
    send("C-g")

    send("C-p")
    send_literal("open a file in vellum")
    send("Enter")
    expect("REFUSED", "Vellum is not configured")

    send("C-p")
    send_literal("inspect tui instrumentation")
    send("Enter")
    expect("TUI instrumentation", "input_classified", "feedback_presented", timeout=8)
    send("C-g")

    send("C-g")
    expect("ACTIVITY & RECEIPTS", "Local provider configuration")
    send("Escape")

    send("]")
    expect("AGENDA")
    send("]")
    expect("CHRONOS")
    send("M-2")
    expect("VELLUM")
    send("M-,")
    expect(
        "CONFIGURATION",
        "surfaces",
        "Overview · Agenda · Chronos · Vellum · Agora",
        "Credentials, grant handles, and provider locators are intentionally omitted.",
    )

    send("C-g")
    expect("ACTIVITY & RECEIPTS")
    send("C-q")
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        result = tmux("has-session", "-t", "breadth", check=False)
        if result.returncode != 0:
            break
        time.sleep(0.025)
    else:
        raise AssertionError("Ctrl-Q did not close the breadth session")
finally:
    tmux("kill-server", check=False)

print("PASS: tmux drove every Pantheon surface and global presentation keybinding")
PYTHON
