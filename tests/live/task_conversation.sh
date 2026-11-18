#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
# Keep Unix-domain endpoints below macOS' short sun_path limit even when the
# caller's TMPDIR expands to a long per-user path.
scratch=$(mktemp -d "/tmp/pantheon-task-conversation.XXXXXX")
tmux_socket="$scratch/tmux.sock"
target="$scratch/target"
link_pid=''
agora_pid=''

cleanup() {
  tmux -S "$tmux_socket" kill-server >/dev/null 2>&1 || true
  for pid in "$agora_pid" "$link_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir -m 700 "$scratch/chronos"

cargo build --manifest-path "$workspace/agora-chronos-adapter/Cargo.toml" \
  --bin task-conversation-service --locked --offline --target-dir "$target"
cargo build --manifest-path "$workspace/agora-tui/Cargo.toml" \
  --package agora-tui-cli --example real_daemon_fixture \
  --locked --offline --target-dir "$target"
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline --target-dir "$target"

profile="$scratch/workspace.json"
cat >"$profile" <<EOF
{
  "version": 1,
  "name": "Task conversation live gate",
  "surfaces": ["chronos", "agora"],
  "agora": {
    "socket": "agora.sock",
    "participant": "operator",
    "requires": ["rooms.read.v2"]
  },
  "task_conversation": {
    "socket": "task-conversation.sock",
    "principal": "11111111111111111111111111111111",
    "requires": ["task-conversation.v1"]
  },
  "chronos": {
    "deployment": "hosted",
    "workstreams": [{"name": "Chronos", "directory": "chronos"}],
    "requires": ["tasks.observe.v1"],
    "moira": [],
    "daimon": [],
    "planning": null
  },
  "moira": null,
  "daimon": null,
  "hermes": null,
  "aegis": null,
  "tellus": null,
  "vellum": null
}
EOF

start_link() {
  "$target/debug/task-conversation-service" \
    "$scratch/link-state" "$scratch/task-conversation.sock" \
    >"$scratch/link.out" 2>"$scratch/link.err" &
  link_pid=$!
  wait_socket "$scratch/task-conversation.sock" "$link_pid" "$scratch/link.err"
}

start_agora() {
  "$target/debug/examples/real_daemon_fixture" "$scratch/agora.sock" \
    >"$scratch/agora.out" 2>"$scratch/agora.err" &
  agora_pid=$!
  wait_socket "$scratch/agora.sock" "$agora_pid" "$scratch/agora.err"
}

wait_socket() {
  path=$1
  pid=$2
  log=$3
  attempt=0
  while [ ! -S "$path" ]; do
    if ! kill -0 "$pid" 2>/dev/null || [ "$attempt" -eq 400 ]; then
      sed -n '1,160p' "$log" >&2
      echo "FAIL: service did not publish $path" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep 0.025
  done
}

drain_link() {
  python3 - "$scratch/task-conversation.sock" <<'PY'
import json, socket, sys
with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as stream:
    stream.settimeout(3)
    stream.connect(sys.argv[1])
    stream.sendall(json.dumps({"operation": "drain"}).encode() + b"\n")
    reply = json.loads(stream.makefile("rb").readline(65537))
assert reply["complete"] and reply["status"] == "draining"
PY
  wait "$link_pid"
  link_pid=''
}

start_link
start_agora

python3 - "$tmux_socket" "$target/debug/pantheon" "$profile" <<'PY'
import shlex, subprocess, sys, time

tmux_socket, pantheon, profile = sys.argv[1:]
command = shlex.join([pantheon, "interactive", "--workspace-profile", profile, "--chronos-allow-writes"])

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def screen():
    return tmux("capture-pane", "-p", "-t", "links").stdout

def expect(*markers, timeout=20):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        visible = screen()
        if all(marker in visible for marker in markers):
            return visible
        time.sleep(0.025)
    raise AssertionError(f"missing {markers!r}:\n{visible}")

def keys(*values):
    tmux("send-keys", "-t", "links", *values)

def literal(value):
    tmux("send-keys", "-t", "links", "-l", value)

def palette(query, expected):
    keys("C-p")
    literal(query)
    expect(expected)
    keys("Enter")

tmux("new-session", "-d", "-x", "180", "-y", "42", "-s", "links", command)
expect("Observed 0 tasks", timeout=40)
keys("n")
expect("Task batch")
literal("linked-task")
keys("Enter")
expect("Review 1 exact task(s)")
keys("Enter")
expect("Applied.")

palette("go to agora", "Go to Agora")
expect("Live Room")
keys("Enter")
expect("Standalone CLI reached the real Agora daemon")

palette("bind selected task to open agora room", "Bind selected task to open Agora room")
expect("Bind Chronos task to Agora room", "task-level", "human gate")
keys("c", "Enter")
expect("BOUND", "room activity remains separate from task progress", timeout=15)
keys("Escape")
expect("Live Room")
keys("C-q")

deadline = time.monotonic() + 15
while tmux("has-session", "-t", "links", check=False).returncode == 0:
    if time.monotonic() >= deadline:
        raise AssertionError(
            "Pantheon did not drain after relationship binding:\n" + screen()
        )
    time.sleep(0.025)
PY

wait "$agora_pid"
agora_pid=''
start_agora
headless=$("$target/debug/pantheon" headless --workspace-profile "$profile")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
peer = next(item for item in report["peers"] if item["peer"] == "task-conversation")
assert peer["status"] == "current"
assert peer["capability"] == "task-conversation.v1"
assert peer["revision"] == 1 and peer["links"] == 1
assert "not task progress" in peer["boundary"]
'
wait "$agora_pid"
agora_pid=''

drain_link
start_link
start_agora
headless=$("$target/debug/pantheon" headless --workspace-profile "$profile")
printf '%s' "$headless" | python3 -c '
import json, sys
peer = next(item for item in json.load(sys.stdin)["peers"] if item["peer"] == "task-conversation")
assert peer["status"] == "current" and peer["revision"] == 1 and peer["links"] == 1
'
wait "$agora_pid"
agora_pid=''
drain_link

echo 'PASS: selected Chronos task + open Agora room -> reviewed durable task-conversation link -> headless parity -> owner restart recovery'
