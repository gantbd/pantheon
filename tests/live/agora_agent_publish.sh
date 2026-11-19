#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/pantheon-agora-publish.XXXXXX")

cleanup() {
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"

cargo build --quiet --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

pantheon="$root/target/debug/pantheon"
request="$scratch/request.json"

python3 - "$request" <<'PY'
from pathlib import Path
import json, sys

Path(sys.argv[1]).write_text(json.dumps({
    "version": 1,
    "operation_id": "11" * 16,
    "room_id": "live-room",
    "expected_revision": 3,
    "logical_time": 41,
    "body": "Bounded agent publication reached the Agora owner",
    "addressed_to": ["operator"],
}, separators=(",", ":")))
PY
chmod 600 "$request"
digest=$(python3 - "$request" <<'PY'
from hashlib import sha256
from pathlib import Path
import sys
print(sha256(Path(sys.argv[1]).read_bytes()).hexdigest())
PY
)
if "$pantheon" agora-message-publish "$scratch/missing.sock" operator "$request" \
  "PUBLISH SOMETHING" >"$scratch/confirmation.json" 2>"$scratch/confirmation.err"; then
  echo "inexact Agora publication confirmation was accepted" >&2
  exit 1
fi

cargo test --quiet --manifest-path "$workspace/agora/Cargo.toml" \
  --package agora-local-daemon --test unix_transport \
  public_composition_client_publishes_observes_and_projects_through_durable_daemon \
  --locked --offline -- --exact
cargo test --quiet --manifest-path "$root/cli/Cargo.toml" \
  --test agora_publish_command --locked --offline

python3 - "$scratch" "$digest" <<'PY'
from pathlib import Path
import json, sys

scratch, digest = Path(sys.argv[1]), sys.argv[2]
confirmation = json.loads((scratch / "confirmation.json").read_text())
assert confirmation["complete"] is False
assert confirmation["status"] == "confirmation_refused"
assert confirmation["required_confirmation"].endswith(digest)
PY

echo "PASS: durable actor publication is exact-confirmed, observable, addressed, and content-redacted"
