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

echo 'PASS: owner Hermes daemon -> same public client -> hosted Workbench -> clean reopen'
