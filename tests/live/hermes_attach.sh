#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
workspace=$(CDPATH= cd -- "$root/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/workbench-hermes.XXXXXX")
pid=
cleanup() { if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi; rm -rf "$tmp"; }
trap cleanup EXIT INT TERM

start_fixture() {
  name=$1
  rm -rf "$tmp/$name-ledger"
  rm -f "$tmp/$name.sock"
  cargo run --quiet --manifest-path "$workspace/hermes-tui/Cargo.toml" \
    -p hermes-tui-cli --example real_daemon_fixture -- \
    "$tmp/$name-ledger" "$tmp/$name.sock" >"$tmp/$name.out" 2>"$tmp/$name.err" &
  pid=$!
  i=0
  while [ ! -S "$tmp/$name.sock" ]; do
    i=$((i+1))
    if [ "$i" -ge 100 ]; then cat "$tmp/$name.err" >&2; exit 1; fi
    sleep 0.05
  done
}

finish_fixture() { wait "$pid"; pid=; }

start_consumer_fixture() {
  name=$1
  rm -rf "$tmp/$name-ledger"
  rm -f "$tmp/$name.sock"
  cargo run --quiet --manifest-path "$workspace/hermes-tui/Cargo.toml" \
    -p hermes-tui-cli --example consumer_daemon_fixture -- \
    "$tmp/$name-ledger" "$tmp/$name.sock" >"$tmp/$name.out" 2>"$tmp/$name.err" &
  pid=$!
  i=0
  while [ ! -S "$tmp/$name.sock" ]; do
    i=$((i+1))
    if [ "$i" -ge 100 ]; then cat "$tmp/$name.err" >&2; exit 1; fi
    sleep 0.05
  done
}

start_fixture observe
printf '%s\n' '{' '"version":1,' '"name":"Hermes attach qualification",' '"surfaces":["hermes"],' "\"hermes\":{\"socket\":\"$tmp/observe.sock\",\"stream\":\"05050505050505050505050505050505\",\"consumer\":\"06060606060606060606060606060606\",\"requires\":[\"ledger.observe.v1\",\"ledger.cursor.observe.v1\"]}," '"agora":null,"moira":null,"daimon":null,"chronos":null,"vellum":null' '}' >"$tmp/profile.json"
output=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- headless --workspace-profile "$tmp/profile.json")
finish_fixture
printf '%s\n' "$output" | grep '"complete": true' >/dev/null
printf '%s\n' "$output" | grep '"peer": "hermes"' >/dev/null
printf '%s\n' "$output" | grep '"capability": "ledger.observe.v1"' >/dev/null
printf '%s\n' "$output" | grep '"streams": 1' >/dev/null
printf '%s\n' "$output" | grep '"retained_events": 1' >/dev/null
printf '%s\n' "$output" | grep '"configured_cursor": {' >/dev/null
printf '%s\n' "$output" | grep '"next_sequence": 1' >/dev/null
printf '%s\n' "$output" | grep '"backlog": 0' >/dev/null

start_fixture refused
if refused=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- \
  hermes-disclose-events "$tmp/refused.sock" 05050505050505050505050505050505 0 1 \
  'disclose stream' 2>"$tmp/refused-client.err"); then
  echo 'FAIL: Pantheon accepted an inexact Hermes disclosure confirmation' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refused" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "hermes"
assert not value["complete"] and value["status"] == "confirmation_refused"
assert value["required_confirmation"] == "DISCLOSE HERMES STREAM 05050505050505050505050505050505 FROM 0 LIMIT 1"
'

start_fixture disclose
disclosed=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- \
  hermes-disclose-events "$tmp/disclose.sock" 05050505050505050505050505050505 0 1 \
  'DISCLOSE HERMES STREAM 05050505050505050505050505050505 FROM 0 LIMIT 1')
finish_fixture
printf '%s' "$disclosed" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "hermes"
assert value["complete"] and value["capability"] == "ledger.events.disclose.v1"
assert value["start"] == 0 and value["limit"] == 1 and value["gap"] is None
assert len(value["events"]) == 1 and value["events"][0]["payload_hex"] == "0a"
'

start_consumer_fixture acknowledge-refused
if refused=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- \
  hermes-acknowledge "$tmp/acknowledge-refused.sock" \
  06060606060606060606060606060606 05050505050505050505050505050505 1 1 \
  'acknowledge' 2>"$tmp/acknowledge-refused-client.err"); then
  echo 'FAIL: Pantheon accepted an inexact Hermes acknowledgement confirmation' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refused" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "hermes"
assert not value["complete"] and value["status"] == "confirmation_refused"
assert value["required_confirmation"] == "ACKNOWLEDGE HERMES CONSUMER 06060606060606060606060606060606 STREAM 05050505050505050505050505050505 FROM 1 THROUGH 1"
'

start_consumer_fixture acknowledge
acknowledged=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- \
  hermes-acknowledge "$tmp/acknowledge.sock" \
  06060606060606060606060606060606 05050505050505050505050505050505 1 1 \
  'ACKNOWLEDGE HERMES CONSUMER 06060606060606060606060606060606 STREAM 05050505050505050505050505050505 FROM 1 THROUGH 1')
finish_fixture
printf '%s' "$acknowledged" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "hermes"
assert value["complete"] and value["capability"] == "ledger.acknowledge.v1"
assert value["status"] == "acknowledged"
assert value["previous_next_sequence"] == 1 and value["next_sequence"] == 2
'

start_consumer_fixture compact-refused
if refused=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- \
  hermes-compact "$tmp/compact-refused.sock" \
  05050505050505050505050505050505 0 1 0 \
  'compact' 2>"$tmp/compact-refused-client.err"); then
  echo 'FAIL: Pantheon accepted an inexact Hermes compaction confirmation' >&2
  exit 1
fi
finish_fixture
printf '%s' "$refused" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "hermes"
assert not value["complete"] and value["status"] == "confirmation_refused"
assert value["required_confirmation"] == "COMPACT HERMES STREAM 05050505050505050505050505050505 RETAINED FROM 0 PUBLISHED EDGE 1 THROUGH 0"
'

start_consumer_fixture compact
compacted=$(cargo run --quiet --manifest-path "$root/Cargo.toml" -p pantheon-cli -- \
  hermes-compact "$tmp/compact.sock" 05050505050505050505050505050505 0 1 0 \
  'COMPACT HERMES STREAM 05050505050505050505050505050505 RETAINED FROM 0 PUBLISHED EDGE 1 THROUGH 0')
finish_fixture
printf '%s' "$compacted" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["shell"] == "workbench" and value["peer"] == "hermes"
assert value["complete"] and value["capability"] == "ledger.compact.v1"
assert value["status"] == "compacted"
assert value["removed"] == 1 and value["retained_from"] == 1
'

echo 'PASS: real Hermes daemon -> observation, disclosure, acknowledgement and compaction owner guards -> Pantheon'
