#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-j1-uadd-sme.XXXXXX)
workbench=${PANTHEON_BIN:-"$repo/target/debug/pantheon"}
fixture="$repo/tests/live/j1_uadd_sme_plan_fixture.exs"
request="$repo/design/requests/j1-uadd-acceptance.sme-plan.json"
security_request="$repo/design/requests/uadd-security-validation.sme-plan.json"
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

if [ "$(stat -f %Lp "$scratch/token")" != 600 ]; then
  echo 'FAIL: fixture token is not mode 0600' >&2
  exit 1
fi

endpoint="127.0.0.1:$(cat "$scratch/port")"
report=$(
  "$workbench" tellus-sme-plan "$endpoint" "$request" \
    --token-file "$scratch/token"
)

printf '%s' "$report" | jq -e '
  .version == 1 and
  .peer == "tellus" and
  .capability == "tellus.sme-plan.v1" and
  .status == "review_required" and
  .complete == false and
  .ensemble_question == "do we have the right ensemble?" and
  [.roles[].role] == [
    "architecture",
    "network",
    "identity",
    "data",
    "adversarial-tester",
    "user-experience",
    "humanism",
    "live-verification"
  ] and
  .orthogonality.sufficient_roles == true and
  .orthogonality.all_orthogonal == false and
  ([.orthogonality.pairs[] | select(.orthogonal == false)] == [{
    "facet1": "human-outcomes",
    "facet2": "human-outcomes",
    "orthogonal": false,
    "role1": "rune:user-experience",
    "role2": "rune:humanism"
  }]) and
  (.boundary | contains("not stakeholder equity"))
' >/dev/null

security_report=$(
  "$workbench" tellus-sme-plan "$endpoint" "$security_request" \
    --token-file "$scratch/token"
)

printf '%s' "$security_report" | jq -e '
  .version == 1 and
  .peer == "tellus" and
  .capability == "tellus.sme-plan.v1" and
  .status == "review_required" and
  .complete == false and
  [.roles[].role] == [
    "security",
    "security.stride",
    "adversarial-tester",
    "live-verification"
  ] and
  .orthogonality.sufficient_roles == true and
  .orthogonality.all_orthogonal == false and
  ([.orthogonality.pairs[] | select(.orthogonal == false)] | length) == 3 and
  any(.orthogonality.pairs[];
    .role1 == "rune:security" and
    .role2 == "rune:security.stride" and
    .orthogonal == false and
    .facet1 == "risk-protection" and
    .facet2 == "risk-protection") and
  any(.orthogonality.pairs[];
    .role1 == "rune:security" and
    .role2 == "rune:adversarial-tester" and
    .orthogonal == false) and
  any(.orthogonality.pairs[];
    .role1 == "rune:security.stride" and
    .role2 == "rune:adversarial-tester" and
    .orthogonal == false) and
  (.boundary | contains("not stakeholder equity"))
' >/dev/null

echo 'PASS: authenticated J1 UADD SME plans return grounded review roles, preserve human/security correlation, and grant no stakeholder authority'
