#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-rune-author-review.XXXXXX)
workbench=${PANTHEON_BIN:-"$repo/target/debug/pantheon"}
author=${RUNE_AUTHOR_BIN:-"$workspace/rune/src/rust/target/debug/rune-author"}
scry=${RUNE_SCRY_BIN:-"$workspace/rune/bin/scry"}
smith=${RUNE_SMITH_BIN:-"$workspace/rune/src/rust/target/debug/smith"}
rustc_sysroot=$(rustc --print sysroot)
rustc=${RUNE_RUSTC_BIN:-"$rustc_sysroot/bin/rustc"}
fixture="$repo/tests/live/j1_uadd_sme_plan_fixture.exs"
architecture_request="$repo/design/requests/rune-author-capability.sme-plan.json"
verification_request="$repo/design/requests/rune-author-capability-verification.sme-plan.json"
author_request="$repo/design/requests/pantheon-rune-author-capability.rune-author.json"
pid=

cleanup() {
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

if [ ! -x "$workbench" ]; then
  cargo build --manifest-path "$repo/Cargo.toml" \
    --package pantheon-cli --bin pantheon --locked --offline
fi
if [ ! -x "$author" ] || [ ! -x "$smith" ]; then
  cargo build --manifest-path "$workspace/rune/src/rust/Cargo.toml" \
    --package rune-author --package smith --locked --offline
fi

(
  cd "$workspace/dendrite"
  PANTHEON_J1_UADD_PORT_FILE="$scratch/port" \
    PANTHEON_J1_UADD_TOKEN_FILE="$scratch/token" \
    MIX_ENV=test mix run "$fixture"
) >"$scratch/dendrite.log" 2>&1 &
pid=$!

attempt=0
while [ ! -s "$scratch/port" ] || [ ! -s "$scratch/token" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -ge 200 ]; then
    sed -n '1,160p' "$scratch/dendrite.log" >&2
    echo 'FAIL: authenticated Dendrite fixture did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

test "$(stat -f %Lp "$scratch/token")" = 600
endpoint="127.0.0.1:$(cat "$scratch/port")"

"$workbench" tellus-sme-plan "$endpoint" "$architecture_request" \
  --token-file "$scratch/token" >"$scratch/architecture.json"
jq -e '
  .status == "review_required" and
  .complete == false and
  [.roles[].role] == ["rigor", "architecture", "network", "identity", "data", "security", "user-experience", "humanism"] and
  .orthogonality.sufficient_roles == true and
  .orthogonality.all_orthogonal == false and
  ([.orthogonality.pairs[] | select(.orthogonal == false)] == [{
    "facet1": "human-outcomes",
    "facet2": "human-outcomes",
    "orthogonal": false,
    "role1": "rune:user-experience",
    "role2": "rune:humanism"
  }]) and
  .lattice.load_id == "pantheon/design/pantheon-concerns" and
  (.boundary | contains("not stakeholder equity"))
' "$scratch/architecture.json" >/dev/null

"$workbench" tellus-sme-plan "$endpoint" "$verification_request" \
  --token-file "$scratch/token" >"$scratch/verification.json"
jq -e '
  .status == "review_required" and
  .complete == false and
  [.roles[].role] == ["network", "distributed-systems", "rust-engineering", "test-harness-engineering", "live-verification"] and
  .orthogonality.sufficient_roles == true and
  .orthogonality.all_orthogonal == false and
  ([.orthogonality.pairs[] | select(.orthogonal == false)] | length) == 2 and
  any(.orthogonality.pairs[];
    .role1 == "rune:network" and
    .role2 == "rune:distributed-systems" and
    .facet1 == "distributed-communication" and
    .facet2 == "distributed-communication" and
    .orthogonal == false) and
  any(.orthogonality.pairs[];
    .role1 == "rune:rust-engineering" and
    .role2 == "rune:test-harness-engineering" and
    .facet1 == "lifecycle-process" and
    .facet2 == "lifecycle-process" and
    .orthogonal == false) and
  (.boundary | contains("not stakeholder equity"))
' "$scratch/verification.json" >/dev/null

RUNE_AUTHOR_SCRY="$scry" \
RUNE_AUTHOR_SMITH="$smith" \
RUNE_AUTHOR_RUSTC="$rustc" \
  "$author" <"$author_request" >"$scratch/author.json"
jq -e '
  .status == "projection_qualified" and
  .machine_state == "projection_qualified" and
  .signal == "projection-prepared" and
  .compatibility.status == "complete" and
  .compatibility.unsupported == [] and
  (.canonical_candidate | length) > 0 and
  (.typed_ir | length) > 0 and
  (.projection | length) > 0
' "$scratch/author.json" >/dev/null

echo 'PASS: Pantheon-owned concern lattice -> authenticated Tellus review cohorts -> exact Rune author qualified private projection'
