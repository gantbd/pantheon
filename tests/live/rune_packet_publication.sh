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
preview=$(
  "$pantheon" rune-packet-publication preview \
    "$scratch/publication-profile.json" "$fixture" "$operation"
)
printf '%s' "$preview" | jq -e '
  .status == "preview_returned" and
  .complete == true and
  .published == false and
  .machine_state == "preview_returned" and
  .signal == "preview-returned" and
  .packet_bytes < 65536 and
  .target.revision == 0 and
  .target.bytes == 12 and
  .observation.mutations == 0
' >/dev/null
test "$(cat "$source_file")" = 'unpublished'
printf '%s' "$preview" | jq '.approval_template' >"$scratch/approval.json"
chmod 600 "$scratch/approval.json"
printf '%s' "$preview" | jq -j '.packet' >"$scratch/expected-packet.json"

published=$(
  "$pantheon" rune-packet-publication publish \
    "$scratch/publication-profile.json" "$fixture" "$operation" "$scratch/approval.json"
)
printf '%s' "$published" | jq -e '
  .status == "durable_published" and
  .complete == true and
  .machine_state == "durable_published" and
  .signal == "durable-published" and
  .prior_revision == 0 and
  .published_revision == 1 and
  .durable_outcome == "durable" and
  .disconnect == "clean"
' >/dev/null
cmp "$scratch/expected-packet.json" "$source_file"
grep -Fq 'pantheon-rune-publication-qualification-fixture' "$source_file"

if stale=$(
  "$pantheon" rune-packet-publication publish \
    "$scratch/publication-profile.json" "$fixture" "$operation" "$scratch/approval.json" 2>&1
); then
  echo 'FAIL: stale approval unexpectedly republished' >&2
  exit 1
fi
printf '%s' "$stale" | grep -Fq 'approval does not exactly bind the freshly recomputed packet'
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

echo 'PASS: Rune -> Scry -> Smith packet -> read-only preview -> exact human gate -> one Vellum edit -> durable save; stale and oversized work refused'
