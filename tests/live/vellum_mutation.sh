#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/workbench-vellum-mutation.XXXXXX")
chmod 700 "$scratch"
source_file="$scratch/source.txt"
socket="$scratch/vellum.sock"
profile="$scratch/workspace.json"
tmux_socket="$scratch/tmux.sock"
target="$scratch/target"

cleanup() {
  tmux -S "$tmux_socket" kill-server >/dev/null 2>&1 || true
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon \
  --locked --offline --target-dir "$target"
workbench="$target/debug/pantheon"

write_profile() {
  printf '%s\n' \
    '{' \
    '  "version": 1,' \
    '  "name": "Vellum mutation qualification",' \
    '  "surfaces": ["workspace"],' \
    '  "vellum": {' \
    '    "deployment": "hosted",' \
    "    \"socket\": \"$socket\"," \
    '    "credential": null,' \
    "    \"roots\": [\"$source_file\"]," \
    '    "requires": ["workspace.inventory.v1", "workspace.edit.v1", "workspace.save.v1"]' \
    '  }' \
    '}' >"$profile"
}

run_journey() {
  mode=$1
  session="vellum-$mode"
  args="interactive --workspace-profile $profile --vellum-allow-edit"
  if [ "$mode" = save ]; then
    args="$args --vellum-allow-save"
  fi
  tmux -S "$tmux_socket" -f /dev/null new-session -d -x 150 -y 28 \
    -s "$session" "$workbench $args"

  expect() {
    marker=$1
    attempt=0
    while [ "$attempt" -lt 240 ]; do
      screen=$(tmux -S "$tmux_socket" capture-pane -p -t "$session")
      if printf '%s' "$screen" | grep -Fq "$marker"; then
        return
      fi
      attempt=$((attempt + 1))
      sleep 0.025
    done
    printf '%s\n' "$screen" >&2
    echo "FAIL: Pantheon Vellum journey omitted $marker" >&2
    exit 1
  }

  expect 'alpha'
  tmux -S "$tmux_socket" send-keys -t "$session" Tab
  expect 'focus=Vellum'
  tmux -S "$tmux_socket" send-keys -t "$session" X
  expect 'Xalpha'
  expect 'Dirty'
  tmux -S "$tmux_socket" send-keys -t "$session" C-s
  if [ "$mode" = save ]; then
    expect 'Clean'
    expect 'SETTLED · Vellum save is current and clean · Next: continue editing'
  else
    expect 'Dirty'
    expect 'Refused'
  fi
  tmux -S "$tmux_socket" send-keys -t "$session" C-q
  if [ "$mode" = deny ]; then
    expect 'Unsaved Vellum edit'
    tmux -S "$tmux_socket" send-keys -t "$session" C-q
  fi
  attempt=0
  while tmux -S "$tmux_socket" has-session -t "$session" 2>/dev/null; do
    if [ "$attempt" -eq 240 ]; then
      echo 'FAIL: Pantheon Vellum journey did not exit' >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

write_profile
printf 'alpha' >"$source_file"
run_journey save
[ "$(cat "$source_file")" = 'Xalpha' ] || {
  echo 'FAIL: Pantheon Vellum save did not publish exact source bytes' >&2
  exit 1
}
[ ! -e "$socket" ]
[ ! -e "$socket.control" ]
echo 'PASS: Pantheon hosted Vellum edit/save returned clean durable state'

printf 'alpha' >"$source_file"
run_journey deny
[ "$(cat "$source_file")" = 'alpha' ] || {
  echo 'FAIL: denied Pantheon Vellum save changed source bytes' >&2
  exit 1
}
[ ! -e "$socket" ]
[ ! -e "$socket.control" ]
echo 'PASS: Pantheon hosted Vellum denied save without persistence authority'
