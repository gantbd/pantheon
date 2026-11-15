#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-operational.XXXXXX")
moira_pid=
daimon_pid=
hermes_pid=
aegis_pid=
dendrite_pid=

cleanup() {
  for pid in "$moira_pid" "$daimon_pid" "$hermes_pid" "$aegis_pid" "$dendrite_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

build_fixture() {
  peer=$1
  package=$2
  cargo build --manifest-path "$workspace/$peer-tui/Cargo.toml" \
    --package "$package" --example real_daemon_fixture \
    --locked --offline
}

build_fixture moira moira-tui-cli
build_fixture daimon daimon-tui-cli
build_fixture hermes hermes-tui-cli
build_fixture aegis aegis-tui-cli
cargo build --manifest-path "$root/Cargo.toml" \
  --package workbench-tui-cli --bin workbench-tui \
  --locked --offline

"$workspace/moira-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/moira.db" "$scratch/moira.sock" >"$scratch/moira.log" 2>&1 &
moira_pid=$!
"$workspace/daimon-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/daimon-store" "$scratch/daimon.sock" >"$scratch/daimon.log" 2>&1 &
daimon_pid=$!
"$workspace/hermes-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/hermes-ledger" "$scratch/hermes.sock" >"$scratch/hermes.log" 2>&1 &
hermes_pid=$!
"$workspace/aegis-tui/target/debug/examples/real_daemon_fixture" \
  "$scratch/aegis-authority" "$scratch/aegis.sock" >"$scratch/aegis.log" 2>&1 &
aegis_pid=$!
(cd "$workspace/dendrite" && \
  DENDRITE_GROUNDING_PORT_FILE="$scratch/grounding.port" MIX_ENV=test \
  mix run scripts/grounding_fixture.exs) >"$scratch/dendrite.log" 2>&1 &
dendrite_pid=$!

for peer in moira daimon hermes aegis; do
  attempt=0
  while [ ! -S "$scratch/$peer.sock" ]; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 100 ]; then
      sed -n '1,120p' "$scratch/$peer.log" >&2
      echo "FAIL: $peer fixture did not become ready" >&2
      exit 1
    fi
    sleep 0.05
  done
done

attempt=0
while [ ! -s "$scratch/grounding.port" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$dendrite_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/dendrite.log" >&2
    echo "FAIL: Grounding fixture did not become ready" >&2
    exit 1
  fi
  sleep 0.05
done
grounding_endpoint="127.0.0.1:$(cat "$scratch/grounding.port")"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Operational peer coexistence qualification",' \
  '  "surfaces": ["overview", "moira", "daimon", "hermes", "aegis", "grounding"],' \
  "  \"moira\": {\"socket\": \"$scratch/moira.sock\", \"requires\": [\"sagas.inspect.v1\"]}," \
  "  \"daimon\": {\"socket\": \"$scratch/daimon.sock\", \"requires\": [\"agents.presence.v1\"]}," \
  "  \"hermes\": {\"socket\": \"$scratch/hermes.sock\", \"requires\": [\"ledger.observe.v1\"]}," \
  "  \"aegis\": {\"socket\": \"$scratch/aegis.sock\", \"requires\": [\"authority.observe.v1\"]}," \
  "  \"grounding\": {\"endpoint\": \"$grounding_endpoint\", \"token\": null, \"requires\": [\"grounding.observe.v1\"]}" \
  '}' >"$scratch/workspace.json"

report=$("$root/target/debug/workbench-tui" \
  headless --workspace-profile "$scratch/workspace.json")

wait "$moira_pid"; moira_pid=
wait "$daimon_pid"; daimon_pid=
wait "$hermes_pid"; hermes_pid=
wait "$aegis_pid"; aegis_pid=
kill "$dendrite_pid"
wait "$dendrite_pid" 2>/dev/null || true
dendrite_pid=

printf '%s' "$report" | python3 -c '
import json
import sys

report = json.load(sys.stdin)
assert report["version"] == 1
assert report["mode"] == "headless"
assert report["complete"]
expected = [
    ("moira", "sagas.inspect.v1"),
    ("daimon", "agents.presence.v1"),
    ("hermes", "ledger.observe.v1"),
    ("aegis", "authority.observe.v1"),
    ("grounding", "grounding.observe.v1"),
]
actual = [(peer["peer"], peer["capability"]) for peer in report["peers"]]
assert actual == expected, actual
assert all(peer["status"] == "current" for peer in report["peers"])
'

echo 'PASS: Moira + Daimon + Hermes + Aegis + Grounding real services -> one Workbench headless composition'
