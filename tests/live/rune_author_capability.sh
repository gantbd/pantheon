#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-rune-author.XXXXXX)
workbench=${PANTHEON_BIN:-"$repo/target/debug/pantheon"}
author=${RUNE_AUTHOR_BIN:-"$workspace/rune/src/rust/target/debug/rune-author"}
scry=${RUNE_SCRY_BIN:-"$workspace/rune/bin/scry"}
smith=${RUNE_SMITH_BIN:-"$workspace/rune/src/rust/target/debug/smith"}
rustc_sysroot=$(rustc --print sysroot)
rustc=${RUNE_RUSTC_BIN:-"$rustc_sysroot/bin/rustc"}
incomplete="$repo/design/requests/j1-uadd-composition.rune-author.json"
qualified="$repo/design/requests/pantheon-rune-author-capability.rune-author.json"

cleanup() {
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$repo/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline
if [ ! -x "$author" ] || [ ! -x "$smith" ]; then
  cargo build --manifest-path "$workspace/rune/src/rust/Cargo.toml" \
    --package rune-author --package smith --locked --offline
fi
for tool in "$author" "$scry" "$smith" "$rustc"; do
  if [ ! -f "$tool" ]; then
    echo "FAIL: required pinned tool is absent: $tool" >&2
    exit 1
  fi
done

digest() {
  shasum -a 256 "$1" | awk '{print "sha256:" $1}'
}

jq -n \
  --arg author "$author" --arg author_sha "$(digest "$author")" \
  --arg scry "$scry" --arg scry_sha "$(digest "$scry")" \
  --arg smith "$smith" --arg smith_sha "$(digest "$smith")" \
  --arg rustc "$rustc" --arg rustc_sha "$(digest "$rustc")" \
  '{
    version: 1,
    confinement: "process_contract",
    rune_author: {path: $author, sha256: $author_sha},
    scry: {path: $scry, sha256: $scry_sha},
    smith: {path: $smith, sha256: $smith_sha},
    rustc: {path: $rustc, sha256: $rustc_sha}
  }' >"$scratch/profile.json"

"$workbench" rune-author "$scratch/profile.json" "$incomplete" \
  >"$scratch/questions.json"
jq -e '
  .status == "questions_pending" and
  .complete == false and
  .machine_state == "questions_returned" and
  .signal == "questions-returned" and
  (.owner_outcome.questions | length) == 15 and
  .confinement.mode == "process_contract" and
  .confinement.os_sandbox == false and
  (.confinement.nonclaim | contains("do not revoke ambient filesystem or network access")) and
  .observation.classification == "questions" and
  .observation.requests == 1 and
  .observation.adapter_errors == 0 and
  .observation.oculus_published == false
' "$scratch/questions.json" >/dev/null

"$workbench" rune-author "$scratch/profile.json" "$qualified" \
  >"$scratch/projection.json"
request_sha=$(digest "$qualified")
jq -e --arg request_sha "$request_sha" '
  .status == "projection_qualified" and
  .complete == true and
  .machine_state == "projection_packet_returned" and
  .signal == "projection-packet-returned" and
  .request_sha256 == $request_sha and
  .owner_outcome.request_sha256 == $request_sha and
  .owner_outcome.compatibility.status == "complete" and
  .owner_outcome.compatibility.unsupported == [] and
  .observation.classification == "projection" and
  .observation.requests == 1 and
  .observation.adapter_errors == 0 and
  (.owner_outcome.canonical_candidate | length) > 0 and
  (.owner_outcome.typed_ir | length) > 0 and
  (.owner_outcome.projection | length) > 0
' "$scratch/projection.json" >/dev/null

printf 'not-json\n' >"$scratch/refused.request"
"$workbench" rune-author "$scratch/profile.json" "$scratch/refused.request" \
  >"$scratch/author-refusal.json"
jq -e '
  .status == "refused" and
  .complete == false and
  .machine_state == "author_refusal_returned" and
  .signal == "author-refusal-returned" and
  .observation.classification == "owner_refusal" and
  .observation.adapter_errors == 0 and
  .owner_outcome.status == "refused"
' "$scratch/author-refusal.json" >/dev/null

jq '.rune_author.sha256 = "sha256:0000000000000000000000000000000000000000000000000000000000000000"' \
  "$scratch/profile.json" >"$scratch/bad-profile.json"
if "$workbench" rune-author "$scratch/bad-profile.json" "$qualified" \
  >"$scratch/adapter-refusal.json" 2>"$scratch/adapter-refusal.stderr"; then
  echo 'FAIL: a mismatched deployment-owned executable digest was admitted' >&2
  exit 1
fi
jq -e '
  .status == "adapter_refused" and
  .complete == false and
  .machine_state == "adapter_refused" and
  .signal == "adapter-refused" and
  .stage == "profile" and
  .confinement.os_sandbox == false and
  .observation.classification == "adapter_refusal" and
  .observation.requests == 0 and
  .observation.adapter_errors == 1 and
  .observation.oculus_published == false
' "$scratch/adapter-refusal.json" >/dev/null

printf '%s\n' \
  '#!/bin/sh' \
  'printf '\''{"version":1,"status":"refused","path":"$","reason":"fixture omits required boundary"}\\n'\''' \
  'exit 2' >"$scratch/incomplete-author"
chmod 700 "$scratch/incomplete-author"
jq \
  --arg path "$scratch/incomplete-author" \
  --arg sha "$(digest "$scratch/incomplete-author")" \
  '.rune_author = {path: $path, sha256: $sha}' \
  "$scratch/profile.json" >"$scratch/incomplete-author-profile.json"
if "$workbench" rune-author "$scratch/incomplete-author-profile.json" "$qualified" \
  >"$scratch/incomplete-response.json" 2>"$scratch/incomplete-response.stderr"; then
  echo 'FAIL: an incomplete Rune-owner response was admitted' >&2
  exit 1
fi
jq -e '
  .status == "adapter_refused" and
  .machine_state == "adapter_refused" and
  .stage == "invalid_response" and
  .observation.classification == "adapter_refusal" and
  .observation.requests == 1 and
  .observation.adapter_errors == 1
' "$scratch/incomplete-response.json" >/dev/null

echo 'PASS: Rune-authored controller -> pinned process contract -> questions, qualified projection, owner refusal, profile refusal, and closed-response refusal'
