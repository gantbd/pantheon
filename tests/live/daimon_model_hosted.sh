#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
# Keep hosted Unix-domain endpoints under macOS's short socket-path ceiling.
scratch=$(mktemp -d "/private/tmp/pdmh.XXXXXX")
aegis_pid=
chronos_pid=
hestia_pid=

cleanup() {
  for pid in "$hestia_pid" "$chronos_pid" "$aegis_pid"; do
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM
chmod 700 "$scratch"
mkdir "$scratch/workstream"
chmod 700 "$scratch/workstream"

cargo build --manifest-path "$workspace/aegis/Cargo.toml" \
  --package aegis-daemon --bin aegis-daemon --locked --offline
cargo build --manifest-path "$workspace/daimon/Cargo.toml" \
  --package daimon-local-model --bin daimon-local-model \
  --package daimon-local-agent --example qualification_chronos --locked --offline
cargo build --manifest-path "$workspace/hestia/Cargo.toml" \
  --package hestia-daemon --bin hestia-daemon \
  --package hestiactl --bin hestiactl --locked --offline
cargo build --manifest-path "$root/Cargo.toml" \
  --package pantheon-cli --bin pantheon --locked --offline

aegis="$workspace/aegis/target/debug/aegis-daemon"
model="$workspace/daimon/target/debug/daimon-local-model"
chronos="$workspace/daimon/target/debug/examples/qualification_chronos"
hestia="$workspace/hestia/target/debug/hestia-daemon"
hestiactl="$workspace/hestia/target/debug/hestiactl"
workbench="$root/target/debug/pantheon"
uid=$(id -u)
gid=$(id -g)

provider_port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
python=$(command -v python3)

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  "  \"socket\": \"$scratch/hestia.sock\"," \
  '  "authority": [9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9],' \
  '  "max_runtimes": 1,' \
  '  "profiles": [{' \
  '    "id": "local-model",' \
  '    "endpoint_reference": "daimon-local-model-loopback",' \
  "    \"executable\": \"$python\"," \
  "    \"arguments\": [\"$root/tests/fixtures/openai_qualification_server.py\", \"$scratch/provider.port\", \"$scratch/provider.count\", \"$provider_port\"]," \
  '    "environment": [],' \
  "    \"readiness_address\": \"127.0.0.1:$provider_port\"," \
  '    "startup_timeout_ms": 5000,' \
  '    "probe_interval_ms": 10,' \
  '    "drain_timeout_ms": 1000' \
  '  }]' \
  '}' >"$scratch/hestia.json"
chmod 600 "$scratch/hestia.json"
"$hestia" "$scratch/hestia.json" >"$scratch/hestia.out" 2>"$scratch/hestia.err" &
hestia_pid=$!
attempt=0
while [ ! -S "$scratch/hestia.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$hestia_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/hestia.err" >&2
    echo 'FAIL: Hestia owner did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

printf '%s\n' \
  '{' \
  '  "authority": [9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9],' \
  '  "runtime": [8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8],' \
  '  "operation": [7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7],' \
  '  "expected_revision": 0,' \
  '  "expected_incarnation": 0,' \
  '  "hosting_profile": "local-model",' \
  '  "model": {"identity":"qwen/qwen3-coder-next-staged","digest":null,"quantization":null,"context_tokens":32768},' \
  '  "resources": {"memory_bytes":1073741824,"accelerator":"cpu","threads":4,"concurrency":1},' \
  '  "sandbox": "Inherited",' \
  '  "egress": "Inherited"' \
  '}' >"$scratch/hestia-start-staged.json"
"$hestiactl" start "$scratch/hestia.sock" "$scratch/hestia-start-staged.json" >"$scratch/hestia-start.out"
grep -q '"Ready"' "$scratch/hestia-start.out"

"$aegis" create "$scratch/authority" "$scratch/aegis.sock" \
  01010101010101010101010101010101 "$uid" "$gid" \
  02020202020202020202020202020202 0303030303030303 administrator \
  >"$scratch/aegis.out" 2>"$scratch/aegis.err" &
aegis_pid=$!
"$chronos" "$scratch/chronos.sock" local-model-task >"$scratch/chronos.out" 2>"$scratch/chronos.err" &
chronos_pid=$!

attempt=0
while [ ! -S "$scratch/aegis.sock" ] || [ ! -S "$scratch/chronos.sock" ]; do
  attempt=$((attempt + 1))
  if ! kill -0 "$aegis_pid" 2>/dev/null || ! kill -0 "$chronos_pid" 2>/dev/null || [ "$attempt" -ge 100 ]; then
    sed -n '1,160p' "$scratch/aegis.err" >&2
    sed -n '1,160p' "$scratch/chronos.err" >&2
    echo 'FAIL: owner prerequisites did not become ready' >&2
    exit 1
  fi
  sleep 0.025
done

printf '%s\n' \
  '{' \
  '  "principal": "02020202020202020202020202020202",' \
  "  \"aegis_socket\": \"$scratch/aegis.sock\"," \
  '  "aegis_expected_epoch": 0,' \
  '  "aegis_grant_now": 1,' \
  '  "aegis_grant_expires_at": 4000000000,' \
  "  \"chronos_socket\": \"$scratch/chronos.sock\"," \
  "  \"daimon_socket\": \"$scratch/model.sock\"," \
  "  \"store_directory\": \"$scratch/store\"," \
  '  "provider": {' \
  '    "provider_id": "04040404040404040404040404040404",' \
  '    "harness_profile_id": "05050505050505050505050505050505",' \
  '    "model": "qwen/qwen3-coder-next-staged",' \
  "    \"address\": \"127.0.0.1:$provider_port\"," \
  "    \"host_header\": \"localhost:$provider_port\"," \
  '    "path": "/v1/chat/completions",' \
  '    "structured_output": false,' \
  '    "structured_tools": true,' \
  '    "sampling_temperature_milli": 0,' \
  '    "connect_timeout_ms": 100,' \
  '    "io_timeout_ms": 5000,' \
  '    "max_request_bytes": 65536,' \
  '    "max_response_bytes": 65536' \
  '  },' \
  '  "task_bindings": [{' \
  '    "external_task_key": "06060606060606060606060606060606",' \
  '    "chronos_task_id": "local-model-task"' \
  '  }],' \
  '  "task_execution": {' \
  '    "task_incarnation": 1,' \
  '    "attempt_number": 1,' \
  '    "budget": {' \
  '      "max_elapsed_millis": 5000,' \
  '      "max_input_tokens": 4096,' \
  '      "max_output_tokens": 1024,' \
  '      "max_tool_calls": 4,' \
  '      "max_context_bytes": 65536,' \
  '      "max_output_bytes": 65536' \
  '    }' \
  '  },' \
  '  "context": [{' \
  '    "reference": "07070707070707070707070707070707",' \
  '    "kind": "chronos_work",' \
  '    "text": "bounded local-model work"' \
  '  }],' \
  '  "messages": [{"role": "system", "content": "answer carefully"}],' \
  '  "io_timeout_ms": 10000,' \
  '  "max_executions": 8,' \
  '  "max_requests_per_connection": 4' \
  '}' >"$scratch/model.json"

printf '%s\n' \
  '{' \
  '  "version": 1,' \
  '  "name": "Hosted local model qualification",' \
  '  "surfaces": ["chronos", "daimon"],' \
  '  "chronos": {' \
  '    "deployment": "hosted",' \
  "    \"workstreams\": [{\"name\": \"Model work\", \"directory\": \"$scratch/workstream\"}]," \
  '    "requires": ["tasks.observe.v1"],' \
  '    "moira": [],' \
  '    "daimon": [],' \
  '    "planning": null' \
  '  },' \
  '  "daimon": {' \
  '    "deployment": "hosted",' \
  "    \"model_socket\": \"$scratch/model.sock\"," \
  '    "requires": ["models.endpoint.attach.v1", "models.provider.configure.v1", "models.provider.qualify.v1", "models.task.invoke.v1"],' \
  '    "host": {' \
  "      \"executable\": \"$model\"," \
  "      \"config\": \"$scratch/model.json\"" \
  '    }' \
  '  },' \
  '  "hestia": {' \
  "    \"socket\": \"$scratch/hestia.sock\"," \
  '    "runtime_bindings": [{' \
  '      "provider_id": "04040404040404040404040404040404",' \
  '      "runtime_id": "08080808080808080808080808080808"' \
  '    }]' \
  '  }' \
  '}' >"$scratch/profile.json"

set +e
headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
headless_status=$?
set -e
if [ "$headless_status" -ne 0 ]; then
  printf '%s\n' "$headless" >&2
  echo 'FAIL: hosted local-model headless observation was incomplete' >&2
  exit 1
fi
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
peer = next(peer for peer in report["peers"] if peer["peer"] == "daimon")
assert peer["status"] == "current"
assert peer["capability"] == "models.endpoint.attach.v1"
assert peer["model_endpoint"]["status"] == "attached"
assert "no execution, provider health, qualification, or authority" in peer["model_endpoint"]["boundary"]
configuration = report["provider_configuration"]
assert configuration["status"] == "current"
catalog = configuration["catalog"]
assert catalog["owner"] == "daimon-local-model"
provider = catalog["providers"][0]
assert provider["model"] == "qwen/qwen3-coder-next-staged"
assert provider["wire"] == "openai_chat_completions"
assert provider["endpoint_scope"] == "loopback" and provider["endpoint_port"] > 0
assert not provider["structured_output"] and provider["structured_tools"]
assert provider.get("qualification") is None
assert "address" not in provider and "path" not in provider and "bearer_token_env" not in provider
hestia = next(peer for peer in report["peers"] if peer["peer"] == "hestia")
assert hestia["status"] == "current"
runtime = hestia["runtimes"][0]["evidence"]
assert runtime["lifecycle"] == "ready" and runtime["readiness"] == "ready"
assert runtime["readiness_basis"] == "tcp_accept"
assert runtime["model_identity"] == "qwen/qwen3-coder-next-staged"
assert runtime["model_matches_provider"] and runtime["invocation_eligible"]
assert runtime["incarnation"] == 1 and runtime["revision"] == 2
assert runtime["boundary"].startswith("Hestia lifecycle/readiness evidence only")
assert report["hosted_shutdown"]["status"] == "clean"
assert report["hosted_shutdown"]["receipts"] == [
    {"peer": "daimon", "instance": None, "disposition": "clean"},
    {"peer": "chronos", "instance": "Model work", "disposition": "clean"},
]
'
[ ! -e "$scratch/model.sock" ]

python3 - "$scratch" "$workbench" "$hestiactl" <<'PYTHON'
from pathlib import Path
import json, shlex, subprocess, sys, time

scratch, binary, hestiactl = sys.argv[1:]
tmux_socket = str(Path(scratch) / "tmux.sock")
profile = str(Path(scratch) / "profile.json")
command = shlex.join([binary, "interactive", "--workspace-profile", profile, "--chronos-allow-writes"])
command += " 2>" + shlex.quote(str(Path(scratch) / "pantheon.err"))

def tmux(*args, check=True):
    return subprocess.run(
        ["tmux", "-S", tmux_socket, "-f", "/dev/null", *args],
        capture_output=True, text=True, check=check, timeout=5,
    )

def expect(*markers, timeout=10):
    deadline = time.monotonic() + timeout
    screen = ""
    while time.monotonic() < deadline:
        screen = tmux("capture-pane", "-p", "-t", "model").stdout
        if all(marker in screen for marker in markers):
            return screen
        time.sleep(0.025)
    diagnostics = []
    for name in ("pantheon.err", "aegis.err", "chronos.err", "provider.err"):
        path = Path(scratch) / name
        if path.exists():
            diagnostics.append(f"{name}:\n{path.read_text(errors='replace')}")
    for path in (Path(scratch) / "store.task-invocations").glob("*.json"):
        diagnostics.append(f"{path.name}:\n{path.read_text(errors='replace')}")
    raise AssertionError(f"Missing {markers!r}:\n{screen}\n" + "\n".join(diagnostics))

try:
    tmux("new-session", "-d", "-x", "200", "-y", "48", "-s", "model", command)
    expect("CHRONOS", "Model work", "revision 0", timeout=20)
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "go to daimon")
    tmux("send-keys", "-t", "model", "Enter")
    expect("DAIMON", "MODEL ENDPOINT ATTACHED", "No execution, provider health, qualification", timeout=20)
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "inspect local provider configuration")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "Local provider configuration",
        "daimon-local-model",
        "qwen/qwen3-coder-next-staged",
        "openai_chat_completions",
        "loopback:",
        "commands, arguments, paths, credential names, and secrets remain redacted",
    )
    tmux("send-keys", "-t", "model", "C-g")
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "edit the selected local model provider")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "Edit local model provider",
        "EDITING",
        "qwen/qwen3-coder-next-staged",
        "remain unprojected",
    )
    tmux("send-keys", "-t", "model", "Tab")
    tmux("send-keys", "-t", "model", "Tab")
    tmux("send-keys", "-t", "model", "C-u")
    tmux("send-keys", "-t", "model", "-l", "qwen/qwen3-coder-next")
    for _ in range(11):
        tmux("send-keys", "-t", "model", "Tab")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "REVIEWING",
        "changed fields      model",
        "qualify after restart true",
        "qwen/qwen3-coder-next",
    )
    tmux("send-keys", "-t", "model", "c")
    expect("ARMED", "reviewed candidate digest")
    tmux("send-keys", "-t", "model", "Enter")
    expect("APPLIED", "restart required", "restart/qualify the provider")
    tmux("send-keys", "-t", "model", "C-g")
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "qualify selected local model provider")
    tmux("send-keys", "-t", "model", "Enter")
    expect("running model profile is stale", "drain/restart")
    tmux("send-keys", "-t", "model", "C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "model", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not drain after provider replacement")
        time.sleep(0.025)

    def hestia(command, operand):
        completed = subprocess.run(
            [hestiactl, command, str(Path(scratch) / "hestia.sock"), str(operand)],
            capture_output=True, text=True, check=True, timeout=10,
        )
        return json.loads(completed.stdout)["result"]

    ready = hestia("observe", "08080808080808080808080808080808")
    drain_path = Path(scratch) / "hestia-drain.json"
    drain_path.write_text(json.dumps({
        "authority": [9] * 16,
        "runtime": [8] * 16,
        "operation": [6] * 16,
        "expected_revision": ready["revision"],
        "expected_incarnation": ready["incarnation"],
    }))
    stopped = hestia("drain", drain_path)["Applied"]
    start_path = Path(scratch) / "hestia-start-final.json"
    start_path.write_text(json.dumps({
        "authority": [9] * 16,
        "runtime": [8] * 16,
        "operation": [5] * 16,
        "expected_revision": stopped["revision"],
        "expected_incarnation": stopped["incarnation"],
        "hosting_profile": "local-model",
        "model": {
            "identity": "qwen/qwen3-coder-next",
            "digest": None,
            "quantization": None,
            "context_tokens": 32768,
        },
        "resources": {
            "memory_bytes": 1073741824,
            "accelerator": "cpu",
            "threads": 4,
            "concurrency": 1,
        },
        "sandbox": "Inherited",
        "egress": "Inherited",
    }))
    restarted = hestia("start", start_path)["Applied"]
    assert restarted["lifecycle"] == "Ready"
    assert restarted["incarnation"] == 2

    tmux("new-session", "-d", "-x", "200", "-y", "48", "-s", "model", command)
    expect("CHRONOS", "Model work", "revision 0", timeout=20)
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "go to daimon")
    tmux("send-keys", "-t", "model", "Enter")
    expect("DAIMON", "MODEL ENDPOINT ATTACHED", timeout=20)
    tmux("send-keys", "-t", "model", "C-p")
    tmux("send-keys", "-t", "model", "-l", "qualify selected local model provider")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "Qualify local model provider",
        "READY",
        "human gate          review",
        "tool-continuation conformance",
    )
    tmux("send-keys", "-t", "model", "c")
    expect("ARMED", "one bounded completion-conformance request")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "QUALIFIED",
        "qualified_completion",
        "17 input",
        "4 output tokens",
        "tool continuation    not_exercised",
        timeout=20,
    )
    tmux("send-keys", "-t", "model", "C-g")
    tmux("send-keys", "-t", "model", "C-p", "C-u")
    tmux("send-keys", "-t", "model", "-l", "create task with local model")
    expect("Create task with local model")
    tmux("send-keys", "-t", "model", "Enter")
    expect("Task batch")
    tmux("send-keys", "-t", "model", "-l", "local-model-task")
    tmux("send-keys", "-t", "model", "Enter")
    expect("Review 1 exact task(s)")
    tmux("send-keys", "-t", "model", "Enter")
    expect(
        "Invoke local model for Chronos task",
        "local-model-task",
        "provider posture      qualified_completion · completion true · observed",
        "runtime health         ready · readiness ready (tcp_accept) · incarnation 2",
        "HESTIA RUNTIME EVIDENCE",
        "qwen/qwen3-coder-next · matches Daimon true",
        "sandbox inherited · egress inherited · fenced false",
        "lifecycle/readiness is Hestia-owned",
        "tool continuation     not_exercised",
        "qualification evidence sha256:",
        "known deviations      tool_continuation_not_exercised",
        "context/tools          1 exact / 0 exact",
        "human gate            review",
        "next safe action      bounded completion only",
        timeout=20,
    )
    tmux("send-keys", "-t", "model", "c", "Enter")
    expect(
        "COMPLETED",
        "bounded qualification complete",
        "17 input · 4 output tokens",
        "owner observed",
        "Agora room and Chronos comment",
        "linkage remain open",
        timeout=20,
    )
    tmux("send-keys", "-t", "model", "r")
    expect("replayed              true", timeout=20)
    tmux("send-keys", "-t", "model", "C-g")
    tmux("send-keys", "-t", "model", "C-q")
    deadline = time.monotonic() + 10
    while tmux("has-session", "-t", "model", check=False).returncode == 0:
        if time.monotonic() >= deadline:
            raise AssertionError("Pantheon did not cleanly drain hosted local-model Daimon")
        time.sleep(0.025)
finally:
    tmux("kill-server", check=False)
PYTHON
[ ! -e "$scratch/model.sock" ]

headless=$("$workbench" headless --workspace-profile "$scratch/profile.json")
printf '%s' "$headless" | python3 -c '
import json, sys
report = json.load(sys.stdin)
assert report["complete"]
provider = report["provider_configuration"]["catalog"]["providers"][0]
qualification = provider["qualification"]
assert qualification["disposition"] == "qualified_completion"
assert qualification["completion"] is True
assert qualification["tool_continuation"] == "not_exercised"
assert qualification["input_tokens"] == 17
assert qualification["output_tokens"] == 4
assert qualification["output_bytes"] == 30
assert qualification["evidence_digest"].startswith("sha256:")
assert qualification["known_deviations"] == ["tool_continuation_not_exercised"]
catalog = report["provider_configuration"]["catalog"]
execution = catalog["task_execution"]
assert execution["task_incarnation"] == 1 and execution["attempt_number"] == 1
assert execution["context_references"] == ["07070707070707070707070707070707"]
invocation = catalog["latest_task_invocation"]
assert invocation["chronos_task_id"] == "local-model-task"
assert invocation["disposition"] == "completed"
assert invocation["input_tokens"] == 17 and invocation["output_tokens"] == 4
assert invocation["output_bytes"] == 30
assert invocation["output_digest"].startswith("sha256:")
assert "output" not in invocation
hestia = next(peer for peer in report["peers"] if peer["peer"] == "hestia")
runtime = hestia["runtimes"][0]["evidence"]
assert hestia["status"] == "current"
assert runtime["model_identity"] == "qwen/qwen3-coder-next"
assert runtime["model_matches_provider"] and runtime["invocation_eligible"]
assert runtime["incarnation"] == 2
assert report["hosted_shutdown"]["status"] == "clean"
'
[ "$(cat "$scratch/provider.count")" = 2 ]
[ ! -e "$scratch/model.sock" ]

python3 - "$scratch" "$hestiactl" <<'PYTHON'
from pathlib import Path
import json, subprocess, sys

scratch, ctl = sys.argv[1:]
socket = str(Path(scratch) / "hestia.sock")
runtime = "08080808080808080808080808080808"
observed = json.loads(subprocess.run(
    [ctl, "observe", socket, runtime], capture_output=True, text=True, check=True,
).stdout)["result"]
request = Path(scratch) / "hestia-final-drain.json"
request.write_text(json.dumps({
    "authority": [9] * 16,
    "runtime": [8] * 16,
    "operation": [4] * 16,
    "expected_revision": observed["revision"],
    "expected_incarnation": observed["incarnation"],
}))
drained = json.loads(subprocess.run(
    [ctl, "drain", socket, str(request)], capture_output=True, text=True, check=True,
).stdout)["result"]["Applied"]
assert drained["lifecycle"] == "Stopped"
shutdown = json.loads(subprocess.run(
    [ctl, "shutdown", socket, "09090909090909090909090909090909"],
    capture_output=True, text=True, check=True,
).stdout)["result"]
assert shutdown == "Clean"
PYTHON
wait "$hestia_pid"
hestia_pid=
[ ! -e "$scratch/hestia.sock" ]

if kill -0 "$chronos_pid" 2>/dev/null; then
  kill -TERM "$chronos_pid"
fi
wait "$chronos_pid" || true
chronos_pid=
kill -TERM "$aegis_pid"
wait "$aegis_pid"
aegis_pid=
echo 'PASS: Hestia-owned local runtime -> guarded exact-field edit/apply -> owner-retained qualification -> Chronos task -> Hestia-correlated human gate -> reviewed replay-fenced invocation -> headless/TUI receipt parity -> two provider calls -> clean dual-owner drain'
