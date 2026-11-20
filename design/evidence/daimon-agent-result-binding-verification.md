# Daimon agent-result binding verification

Date: 2026-11-20

Status: focused owner contract and live observation path qualified; no task
settlement, publication, acceptance, approval or rollout admission claimed.

## Preserved boundary

`AgentResultBindingV1` is a Daimon-owned, content-free observation receipt. It
binds one unique retained prompt turn to its execution/task attempt, authority
preparation, admitted context, exact terminal interval and terminal outcome.
It does not carry result bytes and cannot mutate Chronos, publish to Agora,
settle Moira, answer a human gate or promote a Maybe to Bool.

Pantheon obtains the receipt first, reads only the named event interval, and
then independently requires bounded UTF-8 assistant bytes whose SHA-256 equals
both the terminal event and owner receipt digest. Caller-held open/prompt JSON
is never used as owner evidence.

## Refusal laws exercised

- a new prompt cannot reuse a retained turn identity under another operation;
- zero matching prompts is `UnknownTurn`;
- multiple legacy matching prompts is `TurnConflict`;
- no terminal for the retained prompt is `Busy`;
- multiple terminals is `ProviderReceiptConflict`;
- an indeterminate prompt returns `OutcomeUnknown` and does not scan events;
- missing/mismatched completed view, context receipt or preparation refuses;
- Pantheon rejects interval, terminal outcome or digest disagreement.

## Focused evidence

- `cargo test --manifest-path ../daimon/Cargo.toml --package daimon-wire --test agent_wire_behavior --locked --offline` — 5 passed.
- `completed_turn_exposes_exact_content_free_owner_binding` — passed.
- `active_turn_advances_in_bounded_replay_safe_slices` — passed with the new
  pre-advance `Busy` result assertion.
- `indeterminate_agent_result_commit_reopens_exact_result_without_redispatch` —
  passed with the new pre-reconciliation `OutcomeUnknown` assertion.
- `prompt_turn_identity_cannot_be_reused_under_a_fresh_operation` — passed.
- Pantheon `j1_agent_result` focused library tests — forged FSM gate and
  owner-binding mismatch gate passed.
- `tests/live/j1_agent_result_observe.sh` — passed the guarded real Unix socket
  sequence and asserted exact execution/task-attempt projection.
- Daimon and Daimon TUI workspace-wide `--all-targets` compile checks passed;
  Pantheon and standalone Daimon TUI explicitly classify `Result` as invalid in
  unrelated mutation-command reply paths rather than leaving non-exhaustive or
  wildcard behavior.

## Remaining work

The result is now attributable to the execution and task attempt, but J1 still
needs Moira-owned settlement, the complete Agora publication receipt, current
Chronos `ReadyForReview` evidence, strict composite evaluation and the separate
human/UADD acceptance transition. None is inferred from this receipt.
