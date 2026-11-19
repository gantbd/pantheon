#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-hermes-hosted.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/hermes/Cargo.toml" \
  --package hermes-daemon --bin hermes-daemon --locked --offline
cargo build --manifest-path "$workspace/hermes-tui/Cargo.toml" \
  --package hermes-tui-cli --example seed_operator_ledger --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

daemon="$workspace/hermes/target/debug/hermes-daemon"
workbench="$root/target/debug/pantheon"
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
    '      "subject": "07070707070707070707070707070707",' \
    '      "access": "observer",' \
    '      "operator_stream": null' \
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
socket = Path(scratch) / "hermes.sock"
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
            raise AssertionError("Interactive Pantheon did not drain hosted Hermes")
        time.sleep(0.025)
    while socket.exists():
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon left the hosted Hermes socket behind")
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

"$workspace/hermes-tui/target/debug/examples/seed_operator_ledger" \
  "$scratch/operator-ledger"
cat >"$scratch/operator.json" <<EOF
{
  "version": 1,
  "name": "Hosted Hermes operator qualification",
  "surfaces": ["hermes"],
  "hermes": {
    "deployment": "hosted",
    "socket": "$scratch/operator.sock",
    "stream": "05050505050505050505050505050505",
    "consumer": "06060606060606060606060606060606",
    "requires": ["ledger.observe.v1", "ledger.cursor.observe.v1"],
    "host": {
      "executable": "$daemon",
      "directory": "$scratch/operator-ledger",
      "mode": "open",
      "effective_uid": $uid,
      "effective_gid": $gid,
      "subject": "07070707070707070707070707070707",
      "access": "operator",
      "operator_stream": "05050505050505050505050505050505"
    }
  }
}
EOF

python3 - "$scratch" "$workbench" "$scratch/operator.json" <<'PYTHON'
from pathlib import Path
import json
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
socket = str(Path(scratch) / "operator.sock")
tmux_socket = str(Path(scratch) / "operator-tmux.sock")
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
    tmux("new-session", "-d", "-x", "120", "-y", "24", "-s", "hermes-operator", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "hermes-operator").stdout
        if "Hermes" in screen and "streams: 1" in screen:
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Hermes operator pane not ready:\n{screen}")

    acknowledged = subprocess.run(
        [
            binary,
            "hermes-acknowledge",
            socket,
            "06060606060606060606060606060606",
            "05050505050505050505050505050505",
            "1",
            "1",
            "ACKNOWLEDGE HERMES CONSUMER 06060606060606060606060606060606 STREAM 05050505050505050505050505050505 FROM 1 THROUGH 1",
        ],
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )
    acknowledgement = json.loads(acknowledged.stdout)
    assert acknowledgement["complete"] and acknowledgement["status"] == "acknowledged"
    assert acknowledgement["previous_next_sequence"] == 1
    assert acknowledgement["next_sequence"] == 2

    compacted = subprocess.run(
        [
            binary,
            "hermes-compact",
            socket,
            "05050505050505050505050505050505",
            "0",
            "1",
            "0",
            "COMPACT HERMES STREAM 05050505050505050505050505050505 RETAINED FROM 0 PUBLISHED EDGE 1 THROUGH 0",
        ],
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )
    compaction = json.loads(compacted.stdout)
    assert compaction["complete"] and compaction["status"] == "compacted"
    assert compaction["removed"] == 1 and compaction["retained_from"] == 1

    tmux("send-keys", "-t", "hermes-operator", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "hermes-operator", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon did not drain hosted Hermes operator")
        time.sleep(0.025)
    while Path(socket).exists():
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon left the hosted Hermes operator socket behind")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/operator.sock" ]

operator_reopened=$("$workbench" headless --workspace-profile "$scratch/operator.json")
printf '%s' "$operator_reopened" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = report["peers"][0]
assert peer["peer"] == "hermes" and peer["status"] == "current"
assert peer["configured_stream"]["retained_from"] == 1
assert peer["configured_cursor"]["next_sequence"] == 2
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/operator.sock" ]

cat >"$scratch/consumer.json" <<EOF
{
  "version": 1,
  "name": "Hosted Hermes consumer qualification",
  "surfaces": ["hermes"],
  "hermes": {
    "deployment": "hosted",
    "socket": "$scratch/consumer.sock",
    "stream": "05050505050505050505050505050505",
    "consumer": "06060606060606060606060606060606",
    "requires": ["ledger.consumer.observe.v1"],
    "host": {
      "executable": "$daemon",
      "directory": "$scratch/operator-ledger",
      "mode": "open",
      "effective_uid": $uid,
      "effective_gid": $gid,
      "subject": "07070707070707070707070707070707",
      "access": "consumer",
      "operator_stream": null,
      "consumer_stream": "05050505050505050505050505050505"
    }
  }
}
EOF

python3 - "$scratch" "$workbench" "$scratch/consumer.json" <<'PYTHON'
from pathlib import Path
import json
import shlex
import subprocess
import sys
import time

scratch, binary, profile = sys.argv[1:]
socket = str(Path(scratch) / "consumer.sock")
tmux_socket = str(Path(scratch) / "consumer-tmux.sock")
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
    tmux("new-session", "-d", "-x", "120", "-y", "24", "-s", "hermes-consumer", command)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "hermes-consumer").stdout
        if (
            "Hermes" in screen
            and "access: exact-stream consumer" in screen
            and "configured consumer next sequence: 2" in screen
        ):
            break
        time.sleep(0.025)
    else:
        raise AssertionError(f"Hosted Hermes consumer pane not ready:\n{screen}")

    refused = subprocess.run(
        [
            binary,
            "hermes-compact",
            socket,
            "05050505050505050505050505050505",
            "1",
            "1",
            "1",
            "COMPACT HERMES STREAM 05050505050505050505050505050505 RETAINED FROM 1 PUBLISHED EDGE 1 THROUGH 1",
        ],
        capture_output=True,
        text=True,
        check=False,
        timeout=5,
    )
    assert refused.returncode == 1, refused.stderr
    refusal = json.loads(refused.stdout)
    assert not refusal["complete"] and refusal["status"] == "refused"

    tmux("send-keys", "-t", "hermes-consumer", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "hermes-consumer", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon did not drain hosted Hermes consumer")
        time.sleep(0.025)
    while Path(socket).exists():
        if time.monotonic() >= deadline:
            raise AssertionError("Interactive Pantheon left the hosted Hermes consumer socket behind")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/consumer.sock" ]

consumer_reopened=$("$workbench" headless --workspace-profile "$scratch/consumer.json")
printf '%s' "$consumer_reopened" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = report["peers"][0]
assert peer["peer"] == "hermes" and peer["status"] == "current"
assert peer["configured_stream"]["retained_from"] == 1
assert peer["configured_cursor"]["next_sequence"] == 2
assert report["hosted_shutdown"]["status"] == "clean"
'
[ ! -e "$scratch/consumer.sock" ]

echo 'PASS: selected Hermes observer/operator/consumer daemon -> same public clients and owner guards -> hosted Pantheon -> clean reopen'
