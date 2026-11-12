#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
vellum=$(CDPATH= cd -- "$repo/../vellum" && pwd -P)
chronos=$(CDPATH= cd -- "$repo/../../github.com/gantbd/chronos" && pwd -P)
# Keep both nested Unix socket names below macOS's sockaddr_un path bound even
# when TMPDIR expands to a long per-user directory.
scratch=$(mktemp -d /tmp/wb-live.XXXXXX)
chmod 700 "$scratch"
chronos_dir="$scratch/chronos"
vellum_socket="$scratch/vellum.sock"
vellum_control="$scratch/vellum-control.sock"
source_file="$scratch/source.txt"
chronos_target="$scratch/chronos-target"
vellum_target="$scratch/vellum-target"
workbench_target="$scratch/workbench-target"
chronos_bin="$chronos_target/debug/chronos-service"
vellum_bin="$vellum_target/debug/vellum-daemon"
workbench_bin="$workbench_target/debug/workbench-tui"
chronos_log="$scratch/chronos.log"
vellum_log="$scratch/vellum.log"
chronos_pid=
vellum_pid=

cleanup() {
  if [ -n "$vellum_pid" ] && kill -0 "$vellum_pid" 2>/dev/null; then
    if [ -S "$vellum_control" ]; then
      "$vellum_bin" --shutdown "$vellum_control" >/dev/null 2>&1 || true
    fi
    wait "$vellum_pid" 2>/dev/null || true
  fi
  if [ -n "$chronos_pid" ] && kill -0 "$chronos_pid" 2>/dev/null; then
    "$chronos_bin" --shutdown "$chronos_dir" >/dev/null 2>&1 || true
    wait "$chronos_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

printf 'alpha' >"$source_file"
mkdir "$chronos_dir"
chmod 700 "$chronos_dir"
cargo build --manifest-path "$chronos/Cargo.toml" --package chronos-standalone \
  --bin chronos-service --locked --offline --target-dir "$chronos_target"
cargo build --manifest-path "$vellum/Cargo.toml" --package vellum-daemon \
  --bin vellum-daemon --locked --offline --target-dir "$vellum_target"
cargo build --manifest-path "$repo/Cargo.toml" --bin workbench-tui \
  --locked --offline --target-dir "$workbench_target"

"$chronos_bin" "$chronos_dir" >"$chronos_log" 2>&1 &
chronos_pid=$!
"$vellum_bin" --socket "$vellum_socket" --control "$vellum_control" \
  --allow-edit --allow-save -- "$source_file" >"$vellum_log" 2>&1 &
vellum_pid=$!

attempt=0
while [ "$attempt" -lt 100 ]; do
  if "$chronos_bin" --status "$chronos_dir" 2>/dev/null | grep -q 'ready=true' && \
     "$vellum_bin" --status "$vellum_control" 2>/dev/null | grep -q '"readiness":"ready"'; then
    break
  fi
  if ! kill -0 "$chronos_pid" 2>/dev/null || ! kill -0 "$vellum_pid" 2>/dev/null; then
    sed -n '1,120p' "$chronos_log" >&2
    sed -n '1,120p' "$vellum_log" >&2
    echo 'FAIL: a service exited before readiness' >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 0.05
done
if [ "$attempt" -eq 100 ]; then
  sed -n '1,120p' "$chronos_log" >&2
  sed -n '1,120p' "$vellum_log" >&2
  echo 'FAIL: dual-service readiness timeout' >&2
  exit 1
fi

snapshot=$(
  "$workbench_bin" "$chronos_dir" --vellum-service "$vellum_socket" --snapshot
)
if [ "$snapshot" != 'dual-service snapshot current' ]; then
  echo 'FAIL: workbench did not report exact dual-service attachment' >&2
  exit 1
fi

"$vellum_bin" --shutdown "$vellum_control" >/dev/null
wait "$vellum_pid"
vellum_pid=
"$chronos_bin" --shutdown "$chronos_dir" >/dev/null
wait "$chronos_pid"
chronos_pid=
echo 'PASS: real Chronos + Vellum services -> one workbench composition'
