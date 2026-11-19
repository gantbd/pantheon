#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/private/tmp/pmd.XXXXXX")
fixture_pid=
vellum_tmux_socket="$scratch/vellum-tmux.sock"

cleanup() {
  if [ -n "$fixture_pid" ] && kill -0 "$fixture_pid" 2>/dev/null; then
    kill "$fixture_pid" 2>/dev/null || true
    wait "$fixture_pid" 2>/dev/null || true
  fi
  tmux -S "$vellum_tmux_socket" kill-server >/dev/null 2>&1 || true
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
assert peer["registry_revision"] == 7
assert peer["deliberation"]["admission"]["fitness"] == "yes"
stakeholder = peer["deliberation"]["members"][0]["stakeholder"]
assert stakeholder["representation"] == "authorized_representative"
assert stakeholder["accountable_principal"] == "22" * 16
assert peer["deliberation"]["reports"][0]["dissent_digests"] == ["2c" * 32]
gate = peer["deliberation"]["human_gate"]
assert gate["status"] == "settled"
assert gate["request"]["reviewer"] == "31" * 16
assert gate["request"]["material_revision_digest"] == "34" * 32
assert gate["request"]["decision_packet"]["owner_route"] == "3c" * 16
assert gate["request"]["decision_packet"]["artifact_route"] == "3d" * 16
assert gate["request"]["decision_packet"]["expected_revision"] == 62
assert gate["response"]["disposition"] == "changes_requested"
assert gate["response"]["rationale"]["owner_route"] == "40" * 16
assert gate["response"]["rationale"]["expected_revision"] == 66
assert gate["response"]["authority"]["subject"] == "31" * 16
review = peer["review"]
assert review["independence"] == "not_established"
assert review["correlation"]["first_pass"]["committed_before_cross_review"] == 1
dimensions = {item["dimension"]: item for item in review["correlation"]["dimensions"]}
assert dimensions["execution_principal"]["status"] == "observed"
assert dimensions["grounding"]["status"] == "observed"
assert dimensions["provider"]["status"] == "unknown"
assert review["findings"][0]["id"] == "44" * 16
assert review["findings"][0]["finding_digest"] == "46" * 32
assert "infers no ensemble fitness" in peer["boundary"]
assert "do not establish independence" in peer["boundary"]
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
        "human gate packet:",
        "human gate rationale artifact:",
        "human gate authority:",
        "review correlation: NotEstablished members=1",
        "first pass: committed-before-cross-review=1",
        "correlation execution_principal: observed",
        "correlation provider: unknown",
    )
    tmux("send-keys", "-t", session, *(["Down"] * 40))
    expect("finding 44444444", "finding evidence:", "human-gate=false")
    tmux("send-keys", "-t", session, "C-p", "C-u")
    tmux("send-keys", "-t", session, "-l", "open moira deliberation")
    expect("Open Moira deliberation")
    tmux("send-keys", "-t", session, "Enter")
    expect("Opened Moira deliberation", "right ensemble? Yes")
finally:
    tmux("kill-server", check=False)
PYTHON

kill "$fixture_pid"
wait "$fixture_pid" 2>/dev/null || true
fixture_pid=

printf '%s' 'Approve the bounded Pantheon rollout after human review.' >"$scratch/decision-packet.txt"
cat >"$scratch/vellum-profile.json" <<EOF
{
  "version": 1,
  "name": "Decision packet authoring owner",
  "surfaces": ["vellum"],
  "vellum": {
    "deployment": "hosted",
    "socket": "$scratch/vellum.sock",
    "credential": null,
    "roots": ["$scratch/decision-packet.txt"],
    "requires": ["workspace.inventory.v1", "workspace.edit.v1", "workspace.save.v1"]
  }
}
EOF
python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "vellum-tmux.sock")
profile = str(Path(scratch) / "vellum-profile.json")
session = "decision-packet-authoring"
command = shlex.join([
    binary, "interactive", "--workspace-profile", profile,
    "--vellum-allow-edit", "--vellum-allow-save",
])
error_log = str(Path(scratch) / "vellum-pantheon.err")
command += " 2>" + shlex.quote(error_log) + \
    " || { status=$?; echo PANTHEON_EXIT:$status; cat " + shlex.quote(error_log) + \
    "; sleep 60; exit $status; }"

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def expect(marker, timeout=30):
    deadline = time.monotonic() + timeout
    visible = ""
    while time.monotonic() < deadline:
        result = tmux("capture-pane", "-p", "-t", session, check=False)
        visible = result.stdout
        if result.returncode == 0 and marker in visible:
            return
        time.sleep(0.025)
    detail = Path(error_log).read_text(errors="replace") if Path(error_log).exists() else ""
    raise AssertionError(f"missing {marker!r}:\n{visible}\n{detail}")

tmux("new-session", "-d", "-x", "180", "-y", "32", "-s", session, command)
expect("Approve the bounded Pantheon rollout", timeout=40)
tmux("send-keys", "-t", session, "Tab")
tmux("send-keys", "-t", session, "!")
expect("Service Current / Dirty")
tmux("send-keys", "-t", session, "C-s")
expect("Service Current / Clean")
PYTHON
cat >"$scratch/vellum-reader-profile.json" <<EOF
{
  "version": 1,
  "name": "Decision packet reader",
  "surfaces": ["vellum"],
  "vellum": {"socket": "$scratch/vellum.sock", "credential": null}
}
EOF
"$workbench" headless --workspace-profile "$scratch/vellum-reader-profile.json" \
  --operation-id decision-packet-route >"$scratch/vellum-headless.json"
packet_route=$(python3 - "$scratch/vellum-headless.json" <<'PYTHON'
import json, sys
value = json.load(open(sys.argv[1]))
peer = next(item for item in value["peers"] if item["peer"] == "vellum")
context = peer["context_manifest"][0]
print(context["workspace"], context["document"], context["revision"], context["sha256"].removeprefix("sha256:"))
PYTHON
)
set -- $packet_route
packet_workspace=$1
packet_document=$2
packet_revision=$3
packet_sha256=$4

"$fixture" "$scratch/pending.sock" "$scratch/pending.db" \
  pending-packet "$packet_revision" "$packet_sha256" \
  >"$scratch/pending-fixture.out" 2>"$scratch/pending-fixture.err" &
fixture_pid=$!
attempt=0
while [ ! -S "$scratch/pending.sock" ]; do
  if ! kill -0 "$fixture_pid" 2>/dev/null || [ "$attempt" -eq 400 ]; then
    sed -n '1,160p' "$scratch/pending-fixture.err" >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

cat >"$scratch/pending-profile.json" <<EOF
{
  "version": 1,
  "name": "Moira pending human gate qualification",
  "surfaces": ["agenda", "moira"],
  "moira": {
    "deliberation_socket": "$scratch/pending.sock",
    "deliberation": "01010101010101010101010101010101",
    "requires": ["deliberation.inspect.v1"]
  }
}
EOF

cat >"$scratch/pending-reader-profile.json" <<EOF
{
  "version": 1,
  "name": "Moira pending human gate with selected packet reader",
  "surfaces": ["agenda", "moira", "vellum"],
  "moira": {
    "deliberation_socket": "$scratch/pending.sock",
    "deliberation": "01010101010101010101010101010101",
    "decision_packet_reader": {
      "owner_route": "3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c",
      "artifact_route": "3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d",
      "vellum_workspace": $packet_workspace,
      "vellum_document": $packet_document,
      "max_packet_bytes": 4096
    },
    "requires": ["deliberation.inspect.v1"]
  },
  "vellum": {"socket": "$scratch/vellum.sock", "credential": null}
}
EOF

"$workbench" headless --workspace-profile "$scratch/pending-reader-profile.json" \
  --operation-id moira-pending-gate-live >"$scratch/pending-headless.json"
python3 - "$scratch/pending-headless.json" "$scratch/decision-packet.txt" <<'PYTHON'
import json, sys
value = json.load(open(sys.argv[1]))
peer = next(item for item in value["peers"] if item["peer"] == "moira-deliberation")
assert peer["registry_revision"] == 4
assert peer["deliberation"]["human_gate"]["status"] == "pending"
reader = peer["decision_packet_reader"]
assert reader["status"] == "current"
assert reader["text"] == open(sys.argv[2]).read()
assert "Approve the bounded Pantheon rollout" in reader["text"]
assert "response authority remains unavailable" in reader["boundary"]
entries = value["operator_agenda"]["entries"]
assert len(entries) == 1
entry = entries[0]
assert entry["kind"] == "human_gate"
assert entry["actionability"] == "inspect_only"
assert "verified the exact decision packet" in entry["blocker"]
assert "response remains unavailable" in entry["requested_action"]
assert "grants none" in entry["authority"]
PYTHON

python3 - "$scratch" "$workbench" <<'PYTHON'
from pathlib import Path
import shlex, subprocess, sys, time

scratch, binary = sys.argv[1:]
tmux_socket = str(Path(scratch) / "pending-tmux.sock")
profile = str(Path(scratch) / "pending-reader-profile.json")
session = "moira-pending-gate"
command = shlex.join([binary, "interactive", "--workspace-profile", profile]) + \
    " 2>" + shlex.quote(str(Path(scratch) / "pending-pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    result = tmux("capture-pane", "-p", "-t", session, check=False)
    if result.returncode:
        raise AssertionError(Path(scratch, "pending-pantheon.err").read_text(errors="replace"))
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
                         Path(scratch, "pending-pantheon.err").read_text(errors="replace"))

try:
    tmux("new-session", "-d", "-x", "220", "-y", "40", "-s", session, command)
    expect(
        "AGENDA",
        "Named-human deliberation gate",
        "inspect only",
        "blocked by: Pantheon verified the exact decision packet",
        "agenda presence grants no authority",
    )
    tmux("send-keys", "-t", session, "C-p", "C-u")
    tmux("send-keys", "-t", session, "-l", "open moira deliberation")
    expect("Open Moira deliberation")
    tmux("send-keys", "-t", session, "Enter")
    expect(
        "HUMAN GATE · DECISION PACKET",
        "VERIFIED",
        "Approve the bounded Pantheon rollout",
        "response authority remains unavailable",
    )
finally:
    tmux("kill-server", check=False)
PYTHON

tmux -S "$vellum_tmux_socket" send-keys -t decision-packet-authoring C-q
attempt=0
while tmux -S "$vellum_tmux_socket" has-session -t decision-packet-authoring 2>/dev/null; do
  if [ "$attempt" -eq 400 ]; then
    echo "decision-packet authoring owner did not shut down" >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

echo "qualified: Pantheon verifies an exact Vellum decision packet headlessly and keeps pending gates inspect-only until the Aegis response control exists"
