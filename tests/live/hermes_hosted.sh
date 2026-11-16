#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-hermes-hosted.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/hermes/Cargo.toml" \
  --package hermes-daemon --bin hermes-daemon --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package workbench-tui-cli --bin workbench-tui --locked --offline

daemon="$workspace/hermes/target/debug/hermes-daemon"
workbench="$root/target/debug/workbench-tui"
uid=$(id -u)
gid=$(id -g)

run_profile() {
  mode=$1
  profile="$scratch/$mode.json"
  printf '%s\n' \
    '{' \
    '  "version": 1,' \
    '  "name": "Hosted Hermes qualification",' \
    '  "surfaces": ["hermes"],' \
    '  "hermes": {' \
    '    "deployment": "hosted",' \
    "    \"socket\": \"$scratch/hermes.sock\"," \
    '    "stream": null,' \
    '    "consumer": null,' \
    '    "requires": ["ledger.observe.v1"],' \
    '    "host": {' \
    "      \"executable\": \"$daemon\"," \
    "      \"directory\": \"$scratch/ledger\"," \
    "      \"mode\": \"$mode\"," \
    "      \"effective_uid\": $uid," \
    "      \"effective_gid\": $gid," \
    '      "subject": "07070707070707070707070707070707"' \
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
assert [(peer["peer"], peer["status"]) for peer in report["peers"]] == [("hermes", "current")]
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [{"peer": "hermes", "instance": None, "disposition": "clean"}],
}
'
[ ! -e "$scratch/hermes.sock" ]

reopened=$(run_profile open)
printf '%s' "$reopened" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
assert report["peers"][0]["peer"] == "hermes"
assert report["peers"][0]["status"] == "current"
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/hermes.sock" ]

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
    tmux("new-session", "-d", "-x", "120", "-y", "24", "-s", "hermes", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "hermes").stdout
        if "Hermes" in screen and "streams: 0" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Hermes pane not ready:\n{screen}")

    tmux("send-keys", "-t", "hermes", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "hermes", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Workbench did not drain hosted Hermes")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/hermes.sock" ]

reopened_after_interactive=$(run_profile open)
printf '%s' "$reopened_after_interactive" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
assert report["peers"][0]["peer"] == "hermes"
assert report["peers"][0]["status"] == "current"
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/hermes.sock" ]

echo 'PASS: owner Hermes daemon -> same public client -> hosted headless and interactive Workbench -> clean reopen'
