#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
rune_root=${RUNE_ROOT:-"$root/../rune"}
scratch=$(mktemp -d "/private/tmp/pantheon-bounded-agent-development.XXXXXX")

cleanup() {
  rm -rf "$scratch"
}
trap cleanup EXIT HUP INT TERM

if [ ! -x "$rune_root/src/ocaml/test/run_bounded_agent_development_tests.sh" ]; then
  echo 'FAIL: qualified Rune bounded-agent-development test is unavailable' >&2
  exit 1
fi

cd "$root"
cargo run -p pantheon-cli --bin pantheon --locked --offline -- agent-tools \
  >"$scratch/tools.json"
cargo run -p pantheon-cli --bin pantheon --locked --offline -- \
  agent-role-plan frontier-collaborator \
  rune.machine-path.validate.v1 \
  smith.compatibility.inspect.v1 \
  smith.presentation.prepare.v1 \
  >"$scratch/role.json"

python3 - "$scratch/tools.json" "$scratch/role.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    tools = json.load(source)
with open(sys.argv[2], encoding="utf-8") as source:
    role = json.load(source)

expected = {
    "rune.machine-path.validate.v1",
    "smith.compatibility.inspect.v1",
    "smith.presentation.prepare.v1",
}
owner_tools = {
    entry["semantic_capability"]: entry for entry in tools["owner_tools"]
}
if not expected.issubset(owner_tools):
    raise SystemExit(f"missing bounded owner tools: {sorted(expected - owner_tools.keys())}")
for capability in expected:
    entry = owner_tools[capability]
    if entry["default_admission"] != "denied":
        raise SystemExit(f"{capability} was admitted by catalogue presence")
    if entry["model_arguments"] == "":
        raise SystemExit(f"{capability} omitted its model argument boundary")

gaps = {(entry["component"], entry["surface"]): entry for entry in tools["unavailable"]}
for gap in [
    ("rune", "rune.source.validate.v1"),
    ("smith", "smith.executable-projection.prepare.v1"),
    ("vellum", "workspace.changeset.propose.v1"),
]:
    if gap not in gaps or gaps[gap]["aggregate_fallback"] is not False:
        raise SystemExit(f"missing fail-closed gap {gap}")

rune_gap = gaps[("rune", "rune.source.validate.v1")]
if "private bounded typed-IR buffer" not in rune_gap["missing"]:
    raise SystemExit("Rune validation gap does not retain typed IR privately")
if "Vellum derived-artifact proposal" not in rune_gap["next_boundary"]:
    raise SystemExit("Rune validation gap does not separate Vellum publication authority")
vellum_gap = gaps[("vellum", "workspace.changeset.propose.v1")]
if "loses mutation authority" not in vellum_gap["next_boundary"]:
    raise SystemExit("Vellum gap does not freeze designer mutation before IV&V")

required = {entry["semantic_capability"]: entry for entry in role["required_tools"]}
for capability in expected:
    if required[capability]["candidate_disposition"] != "included":
        raise SystemExit(f"frontier plan did not include {capability}")
if role["operational"] is not False or role["status"] != "blocked_by_owner_surfaces":
    raise SystemExit("partial catalogue was mislabeled operational")
PY

"$rune_root/src/ocaml/test/run_bounded_agent_development_tests.sh"

echo 'PASS: bounded agent development is catalogued as deny-default owner tools, remains non-operational across explicit gaps, and its Rune is fail-closed'
