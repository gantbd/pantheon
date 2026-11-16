#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-moira-reconciliation.XXXXXX")
aegis_pid=
owner_pid=

cleanup() {
  if [ -n "$owner_pid" ] && kill -0 "$owner_pid" 2>/dev/null; then
    kill -TERM "$owner_pid" 2>/dev/null || true
    wait "$owner_pid" 2>/dev/null || true
  fi
  if [ -n "$aegis_pid" ] && kill -0 "$aegis_pid" 2>/dev/null; then
    kill -TERM "$aegis_pid" 2>/dev/null || true
    wait "$aegis_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

if [ "${WORKBENCH_SKIP_BUILD:-0}" != 1 ]; then
  cargo build --manifest-path "$workspace/aegis/Cargo.toml" \
    --package aegis-daemon --bin aegis-daemon --locked --offline
  cargo build --manifest-path "$workspace/moira/Cargo.toml" \
    --package moira-saga-local --bin moira-saga-local \
    --example seed_uncertain_sagas --locked --offline
  cargo build --manifest-path "$workspace/moira-tui/Cargo.toml" \
    --package moira-tui-cli --bin moira-tui --locked --offline
  cargo build --manifest-path "$root/Cargo.toml" \
    --package pantheon-cli --bin pantheon --locked --offline
fi

aegis="$workspace/aegis/target/debug/aegis-daemon"
owner="$workspace/moira/target/debug/moira-saga-local"
seed="$workspace/moira/target/debug/examples/seed_uncertain_sagas"
moira_tui="$workspace/moira-tui/target/debug/moira-tui"
workbench="$root/target/debug/pantheon"
uid=$(id -u)
gid=$(id -g)

"$aegis" create "$scratch/authority" "$scratch/aegis.sock" \
  01010101010101010101010101010101 "$uid" "$gid" \
  02020202020202020202020202020202 0303030303030303 administrator \
  >"$scratch/aegis.out" 2>"$scratch/aegis.err" &
aegis_pid=$!

attempt=0
while [ ! -S "$scratch/aegis.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$aegis_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/aegis.err" >&2
    echo 'FAIL: production Aegis owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

cat >"$scratch/owner.json" <<EOF
{
  "database": "$scratch/moira.db",
  "socket": "$scratch/moira.sock",
  "owner": "31313131313131313131313131313131",
  "mappings": [{
    "effective_uid": $uid,
    "effective_gid": $gid,
    "read": true,
    "mutate": true
  }],
  "io_timeout_ms": 3000,
  "max_requests_per_connection": 32,
  "max_frame_bytes": 65536,
  "max_checkpoint_bytes": 65536,
  "max_database_pages": 128,
  "max_sagas": 64,
  "max_retries_per_saga": 2,
  "max_inspection_page": 64,
  "max_operator_actions": 16,
  "operator": {
    "kind": "aegis",
    "socket": "$scratch/aegis.sock",
    "expected_epoch": 0,
    "grant_ttl_seconds": 300,
    "subject": "02020202020202020202020202020202",
    "sagas": [0, 1]
  }
}
EOF

start_owner() {
  control=$1
  mkfifo "$control"
  exec 3<>"$control"
  "$owner" --supervised "$scratch/owner.json" <"$control" \
    >"$scratch/owner.out" 2>"$scratch/owner.err" &
  owner_pid=$!
  attempt=0
  while [ ! -S "$scratch/moira.sock" ]; do
    attempt=$((attempt + 1))
    if ! kill -0 "$owner_pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
      sed -n '1,160p' "$scratch/owner.err" >&2
      echo 'FAIL: authority-bearing Moira owner did not become ready' >&2
      exit 1
    fi
    sleep 0.025
  done
}

stop_owner() {
  printf x >&3
  exec 3>&-
  wait "$owner_pid"
  owner_pid=
  [ ! -e "$scratch/moira.sock" ]
}

json_field() {
  field=$1
  python3 -c "import json,sys; print(json.load(sys.stdin)['$field'])"
}

start_owner "$scratch/control-first"
seeded=$("$seed" "$scratch/moira.sock" 3)
printf '%s' "$seeded" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value == {"storage_revision": 6, "sagas": [0, 1, 2]}
'

# Standalone prepares; Pantheon confirms and recovers through the same owner guard.
challenge0=$("$moira_tui" reconciliation-view "$scratch/moira.sock" 0 keep-uncertain)
revision0=$(printf '%s' "$challenge0" | json_field storage_revision)
action0=$(printf '%s' "$challenge0" | json_field action)
digest0=$(printf '%s' "$challenge0" | json_field confirmation_digest)
confirmed0=$("$workbench" moira-reconciliation-confirm "$scratch/moira.sock" \
  "$revision0" 0 keep-uncertain "$action0" "$digest0" CONFIRM)
printf '%s' "$confirmed0" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["complete"] and value["status"] == "accepted"
assert value["shell"] == "workbench" and value["peer"] == "moira"
assert value["capability"] == "sagas.reconciliation.confirm.v1"
assert not value["replayed"]
'
recovered0=$("$workbench" moira-reconciliation-recover \
  "$scratch/moira.sock" 0 "$action0")
printf '%s' "$recovered0" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["complete"] and value["status"] == "recovered"
assert value["capability"] == "sagas.reconciliation.recovery.v1"
assert value["saga"] == 0 and value["authority_epoch"] == 0
'

# Pantheon prepares; standalone confirms and recovers through that same guard.
challenge1=$("$workbench" moira-reconciliation-view \
  "$scratch/moira.sock" 1 keep-uncertain)
revision1=$(printf '%s' "$challenge1" | json_field storage_revision)
action1=$(printf '%s' "$challenge1" | json_field action)
digest1=$(printf '%s' "$challenge1" | json_field confirmation_digest)
confirmed1=$("$moira_tui" reconciliation-confirm "$scratch/moira.sock" \
  "$revision1" 1 keep-uncertain "$action1" "$digest1" CONFIRM)
printf '%s' "$confirmed1" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["status"] == "accepted"
assert value["capability"] == "sagas.reconciliation.confirm.v1"
assert not value["replayed"]
'
recovered1=$("$moira_tui" reconciliation-recover \
  "$scratch/moira.sock" 1 "$action1")
printf '%s' "$recovered1" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert value["status"] == "recovered" and value["saga"] == 1
assert value["authority_epoch"] == 0
'

# A current server challenge cannot bypass the owner's selected Aegis scope.
challenge2=$("$moira_tui" reconciliation-view "$scratch/moira.sock" 2 keep-uncertain)
revision2=$(printf '%s' "$challenge2" | json_field storage_revision)
action2=$(printf '%s' "$challenge2" | json_field action)
digest2=$(printf '%s' "$challenge2" | json_field confirmation_digest)
if denied=$("$workbench" moira-reconciliation-confirm "$scratch/moira.sock" \
  "$revision2" 2 keep-uncertain "$action2" "$digest2" CONFIRM 2>"$scratch/denied.err"); then
  echo 'FAIL: unselected saga reconciliation was accepted' >&2
  exit 1
fi
printf '%s' "$denied" | python3 -c '
import json, sys
value = json.load(sys.stdin)
assert not value["complete"] and value["status"] == "refused"
assert value["capability"] == "sagas.reconciliation.confirm.v1"
'
unchanged2=$("$workbench" moira-reconciliation-view \
  "$scratch/moira.sock" 2 keep-uncertain)
[ "$(printf '%s' "$unchanged2" | json_field storage_revision)" = "$revision2" ]
stop_owner

# Pantheon receives only the opaque owner config and proves durable hosted reopen.
cat >"$scratch/profile.json" <<EOF
{
  "version": 1,
  "name": "Moira reconciliation qualification",
  "surfaces": ["moira"],
  "moira": {
    "deployment": "hosted",
    "socket": "$scratch/moira.sock",
    "requires": ["sagas.inspect.v1"],
    "host": {
      "executable": "$owner",
      "config": "$scratch/owner.json"
    }
  }
}
EOF
hosted=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$hosted" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = report["peers"][0]
assert peer["peer"] == "moira" and peer["census"]["total"] == 3
assert report["hosted_shutdown"] == {
    "status": "clean",
    "receipts": [{"peer": "moira", "instance": None, "disposition": "clean"}],
}
'
[ ! -e "$scratch/moira.sock" ]

# Both exact receipts remain recoverable after the hosted owner has reopened and drained.
start_owner "$scratch/control-final"
"$moira_tui" reconciliation-recover "$scratch/moira.sock" 0 "$action0" \
  >"$scratch/recovered-final-0.json"
"$workbench" moira-reconciliation-recover "$scratch/moira.sock" 1 "$action1" \
  >"$scratch/recovered-final-1.json"
python3 - "$scratch/recovered-final-0.json" "$scratch/recovered-final-1.json" <<'PYTHON'
import json, sys
first, second = (json.load(open(path, encoding="utf-8")) for path in sys.argv[1:])
assert first["status"] == second["status"] == "recovered"
assert (first["saga"], second["saga"]) == (0, 1)
assert first["authority_epoch"] == second["authority_epoch"] == 0
PYTHON
stop_owner

echo 'PASS: real Aegis -> selected Moira owner -> cross-shell exact reconciliation -> durable receipt recovery -> refusal without authority -> hosted reopen and clean drain'
