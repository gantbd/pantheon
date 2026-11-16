#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
aegis_tui=$(CDPATH= cd -- "$repo/../aegis-tui" && pwd -P)
aegis=$(CDPATH= cd -- "$repo/../aegis" && pwd -P)
scratch=$(mktemp -d /tmp/wb-aegis.XXXXXX)
chmod 700 "$scratch"
target="$scratch/target"
fixture="$target/debug/examples/real_daemon_fixture"
workbench="$target/debug/pantheon"
daemon="$target/debug/aegis-daemon"
socket="$scratch/aegis.sock"
authority="$scratch/authority"
profile="$scratch/workspace.json"
log="$scratch/aegis.log"
pid=

cleanup() {
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$aegis_tui/Cargo.toml" \
  --package aegis-tui-cli --example real_daemon_fixture \
  --locked --offline --target-dir "$target"
cargo build --manifest-path "$repo/Cargo.toml" \
  --package pantheon-cli --bin pantheon \
  --locked --offline --target-dir "$target"
cargo build --manifest-path "$aegis/Cargo.toml" \
  --package aegis-daemon --bin aegis-daemon \
  --locked --offline --target-dir "$target"

cat >"$profile" <<'JSON'
{
  "version": 1,
  "name": "Aegis attach qualification",
  "surfaces": ["overview", "aegis"],
  "aegis": {
    "socket": "aegis.sock",
    "requires": ["authority.observe.v1"]
  }
}
JSON

start_fixture() {
  rm -f "$socket"
  rm -rf "$authority"
  "$fixture" "$authority" "$socket" >"$log" 2>&1 &
  pid=$!
  attempt=0
  while [ ! -S "$socket" ]; do
    if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 100 ]; then
      sed -n '1,120p' "$log" >&2
      echo 'FAIL: Aegis fixture did not become ready' >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

finish_fixture() {
  wait "$pid"
  pid=
}

start_fixture
snapshot=$("$workbench" headless --workspace-profile "$profile")
finish_fixture
printf '%s' "$snapshot" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["version"] == 1 and report["mode"] == "headless" and report["complete"]
assert len(report["peers"]) == 1
peer = report["peers"][0]
assert peer["peer"] == "aegis"
assert peer["capability"] == "authority.observe.v1"
assert peer["status"] == "current" and peer["epoch"] == 0
assert peer["active_grants"] == 6 and peer["retained_decisions"] == 6
assert peer["page_grants"] == 4 and peer["more_grants"]
assert peer["page_decisions"] == 4 and peer["more_decisions"]
'

start_fixture
if refusal=$("$workbench" aegis-revoke-epoch "$socket" 0 'revoke aegis epoch 0' 2>"$scratch/revoke-confirmation.err"); then
  echo 'FAIL: Pantheon accepted an inexact Aegis revocation confirmation' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refusal" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "aegis"
assert not value["complete"] and value["status"] == "confirmation_refused"
assert value["required_confirmation"] == "REVOKE AEGIS EPOCH 0"
'

start_fixture
if refusal=$("$workbench" aegis-revoke-epoch "$socket" 1 'REVOKE AEGIS EPOCH 1' 2>"$scratch/revoke-stale.err"); then
  echo 'FAIL: Pantheon accepted a stale Aegis revocation epoch' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refusal" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert not value["complete"] and value["status"] == "stale_epoch"
'

resource=abababababababababababababababab
confirmation="AUTHORIZE AEGIS EPOCH 0 hermes $resource observe UNTIL 20"
start_fixture
authorized=$("$workbench" aegis-attenuate-authorize-once "$socket" \
  0 10 30 20 hermes "$resource" observe "$confirmation")
finish_fixture
printf '%s' "$authorized" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "aegis"
assert value["complete"] and value["status"] == "authorized"
assert value["capability"] == "authority.attenuate-authorize-once.v1"
assert value["epoch"] == 0 and value["expires_at"] == 20
assert value["engine"] == "hermes" and value["action"] == "observe"
assert "grant" not in value and "resource" not in value and "session" not in value
'

managed_resource=cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd
parent=11111111111111111111111111111111
child=22222222222222222222222222222222
start_fixture
managed=$(printf '%s\n' \
  "{\"version\":1,\"command\":\"issue\",\"operation\":\"$parent\",\"expected_epoch\":0,\"now\":10,\"expires_at\":40,\"scopes\":[{\"engine\":\"hermes\",\"resource\":\"$managed_resource\",\"actions\":[\"observe\",\"delegate\"]}],\"confirmation\":\"ISSUE AEGIS OP $parent EPOCH 0 UNTIL 40 SCOPES hermes:$managed_resource:observe+delegate\"}" \
  "{\"version\":1,\"command\":\"attenuate\",\"operation\":\"$child\",\"parent\":\"$parent\",\"expected_epoch\":0,\"now\":11,\"expires_at\":30,\"scopes\":[{\"engine\":\"hermes\",\"resource\":\"$managed_resource\",\"actions\":[\"observe\"]}],\"confirmation\":\"ATTENUATE AEGIS OP $child FROM $parent EPOCH 0 UNTIL 30 SCOPES hermes:$managed_resource:observe\"}" \
  "{\"version\":1,\"command\":\"authorize\",\"operation\":\"$child\",\"now\":12,\"engine\":\"hermes\",\"resource\":\"$managed_resource\",\"action\":\"observe\",\"confirmation\":\"AUTHORIZE AEGIS OP $child hermes $managed_resource observe\"}" \
  '{"version":1,"command":"list"}' \
  '{"version":1,"command":"close"}' | \
  "$workbench" aegis-manage-session "$socket")
finish_fixture
printf '%s' "$managed" | python3 -c '
import json, sys
values = [json.loads(line) for line in sys.stdin if line.strip()]
assert [value["status"] for value in values] == [
    "issued", "attenuated", "authorized", "current", "closed"
]
assert values[3]["grants"][1]["parent"] == "11111111111111111111111111111111"
assert values[-1]["terminal"]
rendered = json.dumps(values)
assert "handle" not in rendered and "credential" not in rendered
'

start_fixture
revoked=$("$workbench" aegis-revoke-epoch "$socket" 0 'REVOKE AEGIS EPOCH 0')
finish_fixture
printf '%s' "$revoked" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "aegis"
assert value["complete"] and value["status"] == "accepted"
assert value["capability"] == "authority.revoke-epoch.v1"
assert value["expected_epoch"] == 0 and value["epoch"] == 1
'

start_fixture
python3 - "$scratch" "$workbench" "$profile" <<'PYTHON'
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
        capture_output=True, text=True, check=check, timeout=5,
    )

def expect(*markers):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "workbench").stdout
        if all(marker in screen for marker in markers):
            return
        time.sleep(0.025)
    raise AssertionError(f"Missing {markers!r} in:\n{screen}")

try:
    tmux("new-session", "-d", "-x", "120", "-y", "28", "-s", "workbench", command)
    expect("Aegis: attached")
    tmux("send-keys", "-t", "workbench", "]")
    expect("Aegis", "page: 1", "epoch: 0", "active grants: 6", "grant #0", "decision #3")
    tmux("send-keys", "-t", "workbench", "n")
    expect("page: 2", "grant #4", "grant #5", "decision #4", "decision #5")
    tmux("send-keys", "-t", "workbench", "p")
    expect("page: 1", "grant #0", "decision #0")
    tmux("send-keys", "-t", "workbench", "C-q")
    deadline = time.monotonic() + 5
    while tmux("has-session", "-t", "workbench", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not exit")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
finish_fixture

owner_socket="$scratch/owner.sock"
owner_authority="$scratch/owner-authority"
authority_id=31313131313131313131313131313131
principal_id=32323232323232323232323232323232
session_namespace=3333333333333333
"$daemon" create "$owner_authority" "$owner_socket" "$authority_id" \
  "$(id -u)" "$(id -g)" "$principal_id" "$session_namespace" administrator \
  >"$scratch/owner.log" 2>&1 &
pid=$!
attempt=0
while [ ! -S "$owner_socket" ]; do
  if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 100 ]; then
    sed -n '1,160p' "$scratch/owner.log" >&2
    echo 'FAIL: selected Aegis owner did not become ready' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

abandoned=41414141414141414141414141414141
rebuilt=42424242424242424242424242424242
owner_resource=43434343434343434343434343434343
first=$(printf '%s\n' \
  "{\"version\":1,\"command\":\"issue\",\"operation\":\"$abandoned\",\"expected_epoch\":0,\"now\":10,\"expires_at\":40,\"scopes\":[{\"engine\":\"hermes\",\"resource\":\"$owner_resource\",\"actions\":[\"observe\"]}],\"confirmation\":\"ISSUE AEGIS OP $abandoned EPOCH 0 UNTIL 40 SCOPES hermes:$owner_resource:observe\"}" | \
  "$workbench" aegis-manage-session "$owner_socket")
printf '%s' "$first" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["status"] == "issued" and not value["terminal"]
'

second=$(printf '%s\n' \
  '{"version":1,"command":"list"}' \
  "{\"version\":1,\"command\":\"issue\",\"operation\":\"$rebuilt\",\"expected_epoch\":0,\"now\":11,\"expires_at\":50,\"scopes\":[{\"engine\":\"hermes\",\"resource\":\"$owner_resource\",\"actions\":[\"observe\"]}],\"confirmation\":\"ISSUE AEGIS OP $rebuilt EPOCH 0 UNTIL 50 SCOPES hermes:$owner_resource:observe\"}" \
  "{\"version\":1,\"command\":\"authorize\",\"operation\":\"$rebuilt\",\"now\":12,\"engine\":\"hermes\",\"resource\":\"$owner_resource\",\"action\":\"observe\",\"confirmation\":\"AUTHORIZE AEGIS OP $rebuilt hermes $owner_resource observe\"}" \
  '{"version":1,"command":"close"}' | \
  "$workbench" aegis-manage-session "$owner_socket")
printf '%s' "$second" | python3 -c '
import json, sys
values = [json.loads(line) for line in sys.stdin if line.strip()]
assert [value["status"] for value in values] == ["current", "issued", "authorized", "closed"]
assert values[0]["grants"] == []
assert values[1]["grant"]["operation"] == "42424242424242424242424242424242"
assert "41414141414141414141414141414141" not in json.dumps(values)
'
kill -TERM "$pid"
wait "$pid"
pid=
if [ -e "$owner_socket" ]; then
  echo 'FAIL: selected Aegis owner left its socket behind' >&2
  exit 1
fi

echo 'PASS: real Aegis daemon -> bounded pages, managed private-handle session, authority journey, and revocation -> Pantheon'
