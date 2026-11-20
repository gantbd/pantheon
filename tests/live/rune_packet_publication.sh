#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/tmp/pantheon-rune-publication.XXXXXX")
chmod 700 "$scratch"
source_file="$scratch/packet.json"
socket="$scratch/vellum.sock"
control="$scratch/vellum.control.sock"
credential="$scratch/vellum.credential"
daemon_log="$scratch/vellum.log"
target=${CARGO_TARGET_DIR:-"$root/target"}
operation='vdo1.7172737475767778797a7b7c7d7e7f80'
daemon_pid=''

cleanup() {
  if [ -n "$daemon_pid" ]; then
    kill "$daemon_pid" >/dev/null 2>&1 || true
    wait "$daemon_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$workspace/vellum/Cargo.toml" \
  --package vellum-daemon --bin vellum-daemon --locked --offline --target-dir "$target"
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline --target-dir "$target"
daemon="$target/debug/vellum-daemon"
pantheon="$target/debug/pantheon"
author="$workspace/rune/src/rust/target/debug/rune-author"
scry="$workspace/rune/bin/scry"
smith="$workspace/rune/src/rust/target/debug/smith"
rustc_sysroot=$(rustc --print sysroot)
rustc="$rustc_sysroot/bin/rustc"

for tool in "$author" "$scry" "$smith" "$rustc"; do
  if [ ! -x "$tool" ]; then
    echo "FAIL: qualification requires prebuilt Rune tool: $tool" >&2
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
  }' >"$scratch/rune-profile.json"

printf 'unpublished\n' >"$source_file"
"$daemon" --provision-credential "$credential"
"$daemon" --socket "$socket" --control "$control" --credential "$credential" \
  --allow-edit --allow-save -- "$source_file" >"$daemon_log" 2>&1 &
daemon_pid=$!

attempt=0
while [ ! -S "$socket" ]; do
  if ! kill -0 "$daemon_pid" 2>/dev/null; then
    cat "$daemon_log" >&2
    echo 'FAIL: Vellum owner stopped before readiness' >&2
    exit 1
  fi
  if [ "$attempt" -eq 240 ]; then
    cat "$daemon_log" >&2
    echo 'FAIL: Vellum owner did not publish its endpoint' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

jq -n \
  --arg rune_profile "$scratch/rune-profile.json" \
  --arg socket "$socket" --arg credential "$credential" \
  '{
    version: 1,
    rune_profile: $rune_profile,
    vellum_socket: $socket,
    vellum_credential: $credential,
    vellum_document: 1,
    max_packet_bytes: 65536
  }' >"$scratch/publication-profile.json"

fixture="$root/design/requests/rune-packet-publication-qualification.rune-author.json"
jq -n --slurpfile request "$fixture" '{
  version: 1,
  task: {
    id: "qualify-rune-blueprint-publication",
    prose: "Carry this exact already-reviewed Rune request through the thin in-memory composition into the existing human-gated Vellum publisher.",
    risk: "moderate"
  },
  request: $request[0]
}' >"$scratch/blueprint.json"

# Baseline the former manual handoff: prepare the blueprint, materialize its
# canonical request bytes, then invoke the publisher, which must re-enter the
# Rune owner.  The composed path below must produce the same packet with one
# owner request and zero semantic handoff files.
"$pantheon" rune-blueprint "$scratch/rune-profile.json" \
  "$scratch/blueprint.json" >"$scratch/manual-blueprint.json"
jq -j -cS '.request' "$scratch/blueprint.json" >"$scratch/manual-request.json"
manual_request_sha=$(shasum -a 256 "$scratch/manual-request.json" | awk '{print "sha256:" $1}')
test "$manual_request_sha" = "$(jq -r '.lineage.request_ast.current_sha256' "$scratch/manual-blueprint.json")"
manual_preview=$(
  "$pantheon" rune-packet-publication preview \
    "$scratch/publication-profile.json" "$scratch/manual-request.json" "$operation"
)
printf '%s' "$manual_preview" | jq -e '
  .status == "preview_returned" and
  .observation.rune_owner_requests == 1
' >/dev/null

preview=$(
  "$pantheon" rune-blueprint-publication preview \
    "$scratch/publication-profile.json" "$scratch/blueprint.json" "$operation"
)
printf '%s' "$preview" | jq -e '
  .status == "preview_returned" and
  .complete == true and
  .machine_state == "publication_eligible" and
  .signal == "RiskAdmitted" and
  .blueprint.status == "projection_qualified" and
  .blueprint.risk_policy.publication == "eligible_for_separate_human_approved_vellum_patch" and
  .packet_publication.published == false and
  .packet_publication.machine_state == "preview_returned" and
  .packet_publication.packet_bytes < 65536 and
  .packet_publication.target.revision == 0 and
  .packet_publication.target.bytes == 12 and
  .packet_publication.patch.start == 0 and
  .packet_publication.patch.end == 11 and
  .packet_publication.patch.inserted_bytes == (.packet_publication.packet_bytes - 1) and
  (.packet_publication.patch.inserted_sha256 | startswith("sha256:")) and
  .packet_publication.observation.mutations == 0 and
  .observation.rune_owner_requests == 1 and
  .observation.questions_before == 0 and
  .observation.questions_after == 0 and
  .observation.source.component == "pantheon" and
  .observation.source.capability == "rune.blueprint-publication.v1" and
  .observation.source.request_sha256 == .blueprint.lineage.request_ast.current_sha256 and
  .observation.scope == "one_composed_invocation" and
  .observation.sampling.status == "complete" and
  .observation.disclosure == "content_free" and
  .observation.deterministic_revision_operations == 0 and
  .observation.publication_invocations == 1 and
  .observation.refusals == 0 and
  .observation.refusal_stage == null and
  .observation.semantic_handoff_files_materialized == 0 and
  .observation.manual_handoff_files_avoided == 1 and
  .observation.packet_bytes == .packet_publication.packet_bytes and
  .observation.inserted_bytes == .packet_publication.patch.inserted_bytes and
  .observation.pre_code_findings == "not_observed" and
  .observation.rework_operations == "not_observed" and
  .observation.token_usage == "not_observed" and
  .observation.context_usage == "not_observed" and
  .observation.escaped_defects == "not_observed" and
  (.observation.gaps | index("oculus_owner_receipt")) != null and
  .observation.oculus_published == false
' >/dev/null
test "$(printf '%s' "$manual_preview" | jq -r '.packet_sha256')" = \
  "$(printf '%s' "$preview" | jq -r '.packet_publication.packet_sha256')"
test "$(jq -r '.observation.request_count' "$scratch/manual-blueprint.json")" = 1
manual_duration_ms=$((
  $(jq -r '.observation.duration_ms' "$scratch/manual-blueprint.json") +
  $(printf '%s' "$manual_preview" | jq -r '.observation.duration_ms')
))
composed_duration_ms=$(printf '%s' "$preview" | jq -r '.observation.duration_ms')
echo "VALUE: manual_owner_requests=2 composed_owner_requests=1 manual_semantic_handoff_files=1 composed_semantic_handoff_files=0 manual_duration_ms=$manual_duration_ms composed_duration_ms=$composed_duration_ms"
test "$(cat "$source_file")" = 'unpublished'
printf '%s' "$preview" | jq '.packet_publication.approval_template' >"$scratch/approval.json"
chmod 600 "$scratch/approval.json"
printf '%s' "$preview" | jq -j '.packet_publication.packet' >"$scratch/expected-packet.json"

published=$(
  "$pantheon" rune-blueprint-publication publish \
    "$scratch/publication-profile.json" "$scratch/blueprint.json" "$operation" \
    "$scratch/approval.json"
)
printf '%s' "$published" | jq -e '
  .status == "durable_published" and
  .complete == true and
  .machine_state == "durable_published" and
  .signal == "BlueprintPublished" and
  .packet_publication.status == "durable_published" and
  .packet_publication.prior_revision == 0 and
  .packet_publication.published_revision == 1 and
  .packet_publication.durable_outcome == "durable" and
  .packet_publication.disconnect == "clean"
' >/dev/null
cmp "$scratch/expected-packet.json" "$source_file"
grep -Fq 'pantheon-rune-publication-qualification-fixture' "$source_file"

if stale=$(
  "$pantheon" rune-blueprint-publication publish \
    "$scratch/publication-profile.json" "$scratch/blueprint.json" "$operation" \
    "$scratch/approval.json" 2>&1
); then
  echo 'FAIL: stale approval unexpectedly republished' >&2
  exit 1
fi
printf '%s' "$stale" | grep -Fq '"stage": "no_change"'
cmp "$scratch/expected-packet.json" "$source_file"

if oversized=$(
  "$pantheon" rune-packet-publication preview \
    "$scratch/publication-profile.json" \
    "$root/design/requests/rune-packet-publication.rune-author.json" \
    vdo1.8182838485868788898a8b8c8d8e8f90 2>&1
); then
  echo 'FAIL: oversized packet unexpectedly reached preview' >&2
  exit 1
fi
printf '%s' "$oversized" | grep -Fq '"stage": "packet_limit"'
cmp "$scratch/expected-packet.json" "$source_file"

"$daemon" --shutdown "$control"
wait "$daemon_pid"
daemon_pid=''
[ ! -e "$socket" ]
[ ! -e "$control" ]

# Prove a large current document is restored with a one-byte patch rather than
# a whole-document replacement. The owner is offline while the fixture is
# prepared, so no live workspace state is bypassed.
perl -0pi -e 's/qualification-fixture/qualification-fixturE/' "$source_file"
if cmp -s "$scratch/expected-packet.json" "$source_file"; then
  echo 'FAIL: minimal-patch fixture did not differ' >&2
  exit 1
fi
"$daemon" --socket "$socket" --control "$control" --credential "$credential" \
  --allow-edit --allow-save -- "$source_file" >"$daemon_log" 2>&1 &
daemon_pid=$!
attempt=0
while [ ! -S "$socket" ]; do
  if ! kill -0 "$daemon_pid" 2>/dev/null; then
    cat "$daemon_log" >&2
    echo 'FAIL: Vellum owner stopped before minimal-patch readiness' >&2
    exit 1
  fi
  if [ "$attempt" -eq 240 ]; then
    echo 'FAIL: Vellum owner did not publish the minimal-patch endpoint' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

operation2='vdo1.9192939495969798999a9b9c9d9e9fa0'
patch_preview=$(
  "$pantheon" rune-blueprint-publication preview \
    "$scratch/publication-profile.json" "$scratch/blueprint.json" "$operation2"
)
printf '%s' "$patch_preview" | jq -e '
  .status == "preview_returned" and
  .machine_state == "publication_eligible" and
  .packet_publication.patch.start > 0 and
  .packet_publication.patch.end == (.packet_publication.patch.start + 1) and
  .packet_publication.patch.inserted_bytes == 1 and
  .packet_publication.target.bytes == .packet_publication.packet_bytes and
  .packet_publication.observation.inserted_bytes == 1
' >/dev/null
printf '%s' "$patch_preview" | jq '.packet_publication.approval_template' >"$scratch/patch-approval.json"
chmod 600 "$scratch/patch-approval.json"
patch_published=$(
  "$pantheon" rune-blueprint-publication publish \
    "$scratch/publication-profile.json" "$scratch/blueprint.json" "$operation2" \
    "$scratch/patch-approval.json"
)
printf '%s' "$patch_published" | jq -e '
  .status == "durable_published" and
  .complete == true and
  .packet_publication.prior_revision == 0 and
  .packet_publication.published_revision == 1 and
  .packet_publication.durable_outcome == "durable"
' >/dev/null
cmp "$scratch/expected-packet.json" "$source_file"

"$daemon" --shutdown "$control"
wait "$daemon_pid"
daemon_pid=''
[ ! -e "$socket" ]
[ ! -e "$control" ]

echo 'PASS: prose blueprint -> Rune -> Scry -> Smith -> in-memory composition -> minimal Vellum patch -> exact human gate -> durable save; no-change and oversized work refused'
