#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d "/tmp/wb-vd.XXXXXX")
chmod 700 "$scratch"
source_file="$scratch/source.txt"
socket="$scratch/vellum.sock"
control="$scratch/vellum.control.sock"
credential="$scratch/vellum.credential"
target=${CARGO_TARGET_DIR:-"$scratch/target"}
daemon_log="$scratch/daemon.log"
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
cargo build --manifest-path "$workspace/vellum-tui/Cargo.toml" \
  --package vellum-tui-cli --bin vellum-tui --locked --offline --target-dir "$target"
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline --target-dir "$target"

daemon="$target/debug/vellum-daemon"
standalone="$target/debug/vellum-tui"
workbench="$target/debug/pantheon"

printf 'durable source\n' >"$source_file"
"$daemon" --provision-credential "$credential"
"$daemon" --socket "$socket" --control "$control" --credential "$credential" \
  --allow-save -- "$source_file" >"$daemon_log" 2>&1 &
daemon_pid=$!

attempt=0
while [ ! -S "$socket" ]; do
  if ! kill -0 "$daemon_pid" 2>/dev/null; then
    cat "$daemon_log" >&2
    echo 'FAIL: Vellum durable-save owner stopped before readiness' >&2
    exit 1
  fi
  if [ "$attempt" -eq 240 ]; then
    cat "$daemon_log" >&2
    echo 'FAIL: Vellum durable-save owner did not publish its endpoint' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.025
done

if ! standalone_result=$(
  "$standalone" durable-save-execute "$socket" "$credential" "$operation" 1 0 2>&1
); then
  printf '%s\n' "$standalone_result" >&2
  cat "$daemon_log" >&2
  echo 'FAIL: standalone durable save execution refused' >&2
  exit 1
fi
printf '%s\n' "$standalone_result"
printf '%s' "$standalone_result" | grep -Fq '"capability": "workspace.durable-save.execute.v1"'
printf '%s' "$standalone_result" | grep -Fq '"status": "durable"'
printf '%s' "$standalone_result" | grep -Fq '"disconnect": "clean"'

if ! workbench_result=$(
  "$workbench" vellum-durable-save lookup "$socket" "$credential" "$operation" 1 0 2>&1
); then
  printf '%s\n' "$workbench_result" >&2
  cat "$daemon_log" >&2
  echo 'FAIL: Pantheon durable save lookup refused' >&2
  exit 1
fi
printf '%s\n' "$workbench_result"
printf '%s' "$workbench_result" | grep -Fq '"host": "workbench"'
printf '%s' "$workbench_result" | grep -Fq '"capability": "workspace.durable-save.lookup.v1"'
printf '%s' "$workbench_result" | grep -Fq '"status": "durable"'
printf '%s' "$workbench_result" | grep -Fq '"disconnect": "clean"'

"$daemon" --shutdown "$control"
wait "$daemon_pid"
daemon_pid=''
[ ! -e "$socket" ]
[ ! -e "$control" ]
grep -Fq 'durable source' "$source_file"
echo 'PASS: standalone durable save and Pantheon historical lookup reused Vellum public authority'
