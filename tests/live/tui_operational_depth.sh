#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)

run_gate() {
  name=$1
  printf '\n== Pantheon depth: %s ==\n' "$name"
  sh "$repo/tests/live/$name.sh"
}

# One real, user-visible workflow per operational surface. These gates use
# owner services and public clients; they are not snapshot/render-only checks.
if [ "$#" -eq 0 ]; then
  set -- \
    make_run \
    chronos_mutation \
    dual_service \
    agora_mutation \
    moira_reconciliation \
    daimon_prompt \
    work_item_agent_journey \
    grounding_attach \
    aegis_attach \
    hermes_attach
fi

for gate in "$@"; do
  run_gate "$gate"
done

printf '\nPASS: Pantheon operational depth exercised every configured peer surface\n'
