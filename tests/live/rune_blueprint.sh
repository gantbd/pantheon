#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$repo/.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-rune-blueprint.XXXXXX)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

pantheon=${PANTHEON_BIN:-"$repo/target/debug/pantheon"}
author=${RUNE_AUTHOR_BIN:-"$workspace/rune/src/rust/target/debug/rune-author"}
scry=${RUNE_SCRY_BIN:-"$workspace/rune/bin/scry"}
smith=${RUNE_SMITH_BIN:-"$workspace/rune/src/rust/target/debug/smith"}
rustc_sysroot=$(rustc --print sysroot)
rustc=${RUNE_RUSTC_BIN:-"$rustc_sysroot/bin/rustc"}
qualified="$repo/design/requests/rune-packet-publication-qualification.rune-author.json"

if [ -z "${PANTHEON_BIN:-}" ]; then
  cargo build --manifest-path "$repo/Cargo.toml" \
    --package pantheon-cli --bin pantheon --locked --offline
fi
if [ ! -x "$author" ] || [ ! -x "$smith" ]; then
  cargo build --manifest-path "$workspace/rune/src/rust/Cargo.toml" \
    --package rune-author --package smith --locked --offline
fi

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

jq '{
  version: 1,
  task: {
    id: "qualify-rune-blueprint",
    prose: "Turn this exact reviewed task into legal Rune and Rust without inventing missing semantics.",
    risk: "moderate"
  },
  request: (del(.review, .holes))
}' "$qualified" >"$scratch/blueprint.json"

"$pantheon" rune-blueprint "$scratch/profile.json" "$scratch/blueprint.json" \
  >"$scratch/questions.json"

jq -e '
  .capability == "rune.blueprint.v1" and
  .status == "questions_pending" and
  .complete == false and
  .task.risk == "moderate" and
  .task.prose_bytes > 0 and
  (.task.prose_sha256 | startswith("sha256:")) and
  .lineage.prose_lst.semantic_status == "retained_not_inferred" and
  .lineage.request_ast.base_sha256 == .lineage.request_ast.current_sha256 and
  [.questions[].path] == ["$.holes", "$.review"] and
  [.answer_template.answers[].path] == ["$.holes", "$.review"] and
  [.answer_template.answers[].value] == [null, null] and
  .observation.request_count == 1 and
  .observation.questions_before == 2 and
  .observation.questions_after == 2 and
  .observation.answers_applied == 0 and
  .observation.token_usage == "not_observed" and
  .observation.oculus_published == false and
  .risk_policy.declared_risk == "moderate" and
  .risk_policy.classification_authority == "caller_declared_not_verified" and
  .risk_policy.rune_baseline_weakened == false and
  .risk_policy.publication == "not_eligible_until_projection_complete"
' "$scratch/questions.json" >/dev/null

base=$(jq -r '.lineage.request_ast.base_sha256' "$scratch/questions.json")
review=$(jq '.review' "$qualified")
jq -n --arg base "$base" --argjson review "$review" '{
  version: 1,
  base_request_sha256: $base,
  answers: [
    {
      path: "$.holes",
      category: "completeness",
      question: "Which semantic holes remain unresolved? Supply an explicit array, including an empty one.",
      value: []
    },
    {
      path: "$.review",
      category: "authority",
      question: "Who reviewed this exact request, and what was the ensemble-fit answer?",
      value: $review
    }
  ]
}' >"$scratch/answers.json"

"$pantheon" rune-blueprint "$scratch/profile.json" "$scratch/blueprint.json" \
  "$scratch/answers.json" >"$scratch/projection.json"

jq -e '
  .status == "projection_qualified" and
  .complete == true and
  .questions == [] and
  .answer_template == null and
  .revised_blueprint.request.holes == [] and
  .revised_blueprint.request.review.ensemble_fit == "yes" and
  .observation.request_count == 2 and
  .observation.questions_before == 2 and
  .observation.questions_after == 0 and
  .observation.answers_applied == 2 and
  .observation.deterministic_revision_operations == 2 and
  .risk_policy.publication == "eligible_for_separate_human_approved_vellum_patch" and
  .owner_outcome.owner_outcome.compatibility.status == "complete" and
  .owner_outcome.owner_outcome.compatibility.unsupported == [] and
  (.owner_outcome.owner_outcome.canonical_candidate | length) > 0 and
  (.owner_outcome.owner_outcome.typed_ir | length) > 0 and
  (.owner_outcome.owner_outcome.projection | length) > 0
' "$scratch/projection.json" >/dev/null

jq '{
  version: 1,
  task: {
    id: "moderate-risk-needs-dissent",
    prose: "Do not let a moderate-risk projection erase the limits of its review.",
    risk: "moderate"
  },
  request: (.review.dissent = [])
}' "$qualified" >"$scratch/no-dissent.json"
"$pantheon" rune-blueprint "$scratch/profile.json" "$scratch/no-dissent.json" \
  >"$scratch/risk-question.json"
jq -e '
  .status == "questions_pending" and
  .complete == false and
  [.questions[].path] == ["$.review.dissent"] and
  .questions[0].category == "risk" and
  .owner_outcome.status == "projection_qualified" and
  .owner_outcome.complete == true and
  .risk_policy.publication == "not_eligible_until_projection_complete" and
  .risk_policy.rune_baseline_weakened == false
' "$scratch/risk-question.json" >/dev/null

jq '{
  version: 1,
  task: {
    id: "ensemble-not-yet-fit",
    prose: "Retain an explicit no until the accountable reviewers agree that the ensemble is fit.",
    risk: "moderate"
  },
  request: (.review.ensemble_fit = "no")
}' "$qualified" >"$scratch/no-ensemble-fit.json"
"$pantheon" rune-blueprint "$scratch/profile.json" \
  "$scratch/no-ensemble-fit.json" >"$scratch/ensemble-question.json"
jq -e '
  .status == "questions_pending" and
  .complete == false and
  [.questions[].path] == ["$.review.ensemble_fit"] and
  .questions[0].category == "ensemble" and
  .questions[0].question == "What must change before this ensemble is fit for this exact design?" and
  .owner_outcome.status == "questions_pending" and
  .owner_outcome.complete == false and
  .risk_policy.publication == "not_eligible_until_projection_complete" and
  .risk_policy.rune_baseline_weakened == false
' "$scratch/ensemble-question.json" >/dev/null

jq '.base_request_sha256 = "sha256:0000000000000000000000000000000000000000000000000000000000000000"' \
  "$scratch/answers.json" >"$scratch/stale.json"
if "$pantheon" rune-blueprint "$scratch/profile.json" "$scratch/blueprint.json" \
  "$scratch/stale.json" >"$scratch/refused.json" 2>"$scratch/refused.stderr"; then
  echo 'FAIL: stale blueprint answers were admitted' >&2
  exit 1
fi
jq -e '
  .status == "refused" and
  .stage == "answers" and
  (.detail | contains("exact current Rune request"))
' "$scratch/refused.json" >/dev/null

echo 'PASS: prose -> typed Rune questions -> question-bound revision -> Scry -> Smith; stale answers and unfit ensemble refused'
