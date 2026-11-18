#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pmd.XXXXXX")
fixture_pid=

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --manifest-path "$workspace/moira-tui/Cargo.toml" \
  --package moira-tui-cli --example deliberation_fixture --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

fixture="$workspace/moira-tui/target/debug/examples/deliberation_fixture"
workbench="$root/target/debug/pantheon"
"$fixture" "$scratch/deliberation.sock" "$scratch/deliberation.db" \
  >"$scratch/fixture.out" 2>"$scratch/fixture.err" &
fixture_pid=$!

attempt=0
while [ ! -S "$scratch/deliberation.sock" ]; do
  if ! kill -0 "$fixture_pid" 2>/dev/null || [ "$attempt" -eq 400 ]; then
    sed -n '1,160p' "$scratch/fixture.err" >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Moira deliberation qualification",
  "surfaces": ["moira"],
  "moira": {
    "deliberation_socket": "$scratch/deliberation.sock",
    "deliberation": "01010101010101010101010101010101",
    "requires": ["deliberation.inspect.v1"]
  }
}
EOF

"$workbench" headless --workspace-profile "$scratch/profile.json" \
  --operation-id moira-deliberation-live >"$scratch/headless.json"
python3 - "$scratch/headless.json" <<'PYTHON'
import json, sys
value = json.load(open(sys.argv[1]))
peer = next(item for item in value["peers"] if item["peer"] == "moira-deliberation")
assert peer["capability"] == "deliberation.inspect.v1"
assert peer["status"] == "current"
assert peer["registry_revision"] == 5
assert peer["deliberation"]["admission"]["fitness"] == "yes"
stakeholder = peer["deliberation"]["members"][0]["stakeholder"]
assert stakeholder["representation"] == "authorized_representative"
assert stakeholder["accountable_principal"] == "22" * 16
assert peer["deliberation"]["reports"][0]["dissent_digests"] == ["2c" * 32]
gate = peer["deliberation"]["human_gate"]
assert gate["status"] == "settled"
assert gate["request"]["reviewer"] == "31" * 16
assert gate["request"]["material_revision_digest"] == "34" * 32
assert gate["response"]["disposition"] == "changes_requested"
assert gate["response"]["authority"]["subject"] == "31" * 16
assert "infers no ensemble fitness" in peer["boundary"]
PYTHON

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
session = "moira-deliberation"
command = shlex.join([binary, "interactive", "--workspace-profile", profile]) + \
    " 2>" + shlex.quote(str(Path(scratch) / "pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    result = tmux("capture-pane", "-p", "-t", session, check=False)
    if result.returncode:
        raise AssertionError(Path(scratch, "pantheon.err").read_text(errors="replace"))
    return result.stdout

def expect(*markers, timeout=30):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"missing {markers!r}:\n{visible}\n" +
                         Path(scratch, "pantheon.err").read_text(errors="replace"))

try:
    tmux("new-session", "-d", "-x", "220", "-y", "40", "-s", session, command)
    expect(
        "Moira deliberation",
        "right ensemble? Yes",
        "stakeholder AuthorizedRepresentative",
        "stakeholder standing: accountable=22222222222222222222222222222222",
        "evidence=1 dissent=1",
        "human gate: changes_requested",
        "human gate material:",
        "human gate authority:",
    )
    tmux("send-keys", "-t", session, "C-p", "C-u")
    tmux("send-keys", "-t", session, "-l", "open moira deliberation")
    expect("Open Moira deliberation")
    tmux("send-keys", "-t", session, "Enter")
    expect("Opened Moira deliberation", "right ensemble? Yes")
finally:
    tmux("kill-server", check=False)
PYTHON

echo "qualified: Pantheon composes exact Moira ensemble standing and settled human-gate evidence in headless and TUI paths"
