#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-aegis-hosted.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/aegis/Cargo.toml" \
  --package aegis-daemon --bin aegis-daemon --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

daemon="$workspace/aegis/target/debug/aegis-daemon"
workbench="$root/target/debug/pantheon"
uid=$(id -u)
gid=$(id -g)

run_profile() {
  mode=$1
  profile="$scratch/$mode.json"
  printf '%s\n' \
    '{' \
    '  "version": 1,' \
    '  "name": "Hosted Aegis qualification",' \
    '  "surfaces": ["aegis"],' \
    '  "aegis": {' \
    '    "deployment": "hosted",' \
    "    \"socket\": \"$scratch/aegis.sock\"," \
    '    "requires": ["authority.observe.v1"],' \
    '    "host": {' \
    "      \"executable\": \"$daemon\"," \
    "      \"directory\": \"$scratch/authority\"," \
    "      \"mode\": \"$mode\"," \
    '      "authority": "01010101010101010101010101010101",' \
    "      \"effective_uid\": $uid," \
    "      \"effective_gid\": $gid," \
    '      "principal": "02020202020202020202020202020202",' \
    '      "session_namespace": "0303030303030303",' \
    '      "role": "administrator"' \
    '    }' \
    '  }' \
    '}' >"$profile"
  "$workbench" headless --workspace-profile "$profile"
}

created=$(run_profile create)
printf '%s' "$created" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = report["peers"][0]
assert (peer["peer"], peer["status"], peer["epoch"]) == ("aegis", "current", 0)
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [{"peer": "aegis", "instance": None, "disposition": "clean"}],
}
'
[ ! -e "$scratch/aegis.sock" ]

reopened=$(run_profile open)
printf '%s' "$reopened" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
assert report["peers"][0]["epoch"] == 1
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/aegis.sock" ]

python3 - "$scratch" "$workbench" "$scratch/open.json" <<'PYTHON'
from pathlib import Path
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
command = shlex.join([binary, "interactive", "--workspace-profile", profile])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True,
        text=True,
        check=check,
        timeout=5,
    )

try:
    tmux("new-session", "-d", "-x", "120", "-y", "24", "-s", "aegis", command)
    def screen():
        return tmux("capture-pane", "-p", "-t", "aegis").stdout

    def expect(*markers, timeout=10):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            visible = screen()
            if all(marker in visible for marker in markers):
                return visible
            time.sleep(0.025)
        history = tmux("capture-pane", "-p", "-S", "-100", "-t", "aegis").stdout
        raise AssertionError(f"Hosted Aegis markers {markers!r} not ready:\n{history}")

    def keys(*values):
        tmux("send-keys", "-t", "aegis", *values)

    def text(value):
        tmux("send-keys", "-t", "aegis", "-l", value)

    def palette(query, marker):
        keys("C-p")
        text(query)
        expect(marker)
        keys("Enter")

    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        visible = screen()
        if "Aegis" in visible and "epoch: 2" in visible:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Aegis pane not ready:\n{visible}")

    parent = "01010101010101010101010101010101"
    child = "02020202020202020202020202020202"
    resource = "07070707070707070707070707070707"
    palette("manage retained aegis authority", "Manage retained Aegis authority")
    expect("Aegis session ready")
    text(f"issue {parent} 2 5 30 hermes:{resource}:observe+delegate")
    keys("Enter")
    expect("exact confirmation: ISSUE")
    keys("c", "Enter")
    expect("retained grants: 1 / 8", timeout=15)

    keys("e")
    text(f"attenuate {child} {parent} 2 6 20 hermes:{resource}:observe")
    keys("Enter")
    expect("exact confirmation: ATTENUATE")
    keys("c", "Enter")
    expect("retained grants: 2 / 8", timeout=15)

    keys("e")
    text(f"authorize {child} 7 hermes {resource} observe")
    keys("Enter")
    expect("exact confirmation: AUTHORIZE")
    keys("c", "Enter")
    expect("authorized hermes observe until 20", timeout=15)

    palette("inspect tui instrumentation", "Inspect TUI instrumentation")
    expect(
        "0 in flight · 0 budget violations",
        "aegis.authority.manage",
        "peer_receipt",
    )
    keys("Escape")
    time.sleep(0.1)
    keys("Escape")
    expect("Closed Aegis managed session; private grants destroyed")

    keys("C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "aegis", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon did not drain hosted Aegis")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/aegis.sock" ]

reopened_after_interactive=$(run_profile open)
printf '%s' "$reopened_after_interactive" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
assert report["peers"][0]["epoch"] == 3
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/aegis.sock" ]

echo 'PASS: owner Aegis daemon -> hosted Pantheon -> retained N authority work -> instrumented settlement -> clean reopen'
