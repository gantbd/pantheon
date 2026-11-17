# User-journey execution plan

Status: active

Governing blueprint: [`use-cases-and-user-journeys.md`](use-cases-and-user-journeys.md)

This plan turns the product journeys into bounded delivery increments. A checked
item means the named owner contracts, refusal/uncertainty behavior, interactive
and headless projections, and executable evidence all exist. Presentation alone
does not close an item.

## Delivery rules

1. Preserve sovereign peer semantics and authority.
2. Deliver vertical journeys rather than disconnected screens.
3. Retain stable operation identity, exact replay/conflict, revisions, receipts,
   and honest uncertainty at every mutation boundary.
4. Give every action observable success/failure and a next safe action.
5. Require a named-human go/no-go for final production rollout.
6. Pause any other material decision when an admitted human-advocate finding
   requires named-human adjudication.
7. Qualify TUI, CLI/headless, restart/recovery, and accessibility behavior before
   calling a journey complete.

## Dogfood evidence and user-advocacy ledger

This ledger records experience gathered while using the component ensemble to
deliver Pantheon. It is product input, not a substitute for journey acceptance.

### 2026-11-17 — local-provider delegation

- Gemma 3 27B through Ollama completed the grounded chatbot journey across a
  fresh Dendrite, Daimon, Moira, Agora, and Vellum composition.
- Qwen3 Coder Next through LM Studio completed Daimon's bounded provider smoke
  and, after correcting continuation capability accounting, Moira's grounded
  native-tool agent journey. The source turn's authorized tools remain durable
  evidence while exhausted tools are no longer offered to the next turn.
- The exercise required endpoint/model discovery, owner configuration, service
  startup, qualification commands, and interpretation of low-level refusal
  states outside Pantheon. The current Configuration overlay explains those
  steps but does not perform them; this is an operational-surface gap.
- Provider onboarding must therefore be a first-class owner-backed UI journey:
  select an explicitly discovered or entered local endpoint; list owner-reported
  models/capabilities; preflight the redacted profile and restart boundary; run
  bounded completion and tool-continuation checks; retain the exact result and
  deviations; then select only qualified profiles for a task/session.
- Feedback must distinguish unreachable endpoint, unknown model, incompatible
  schema, repeated tool call after budget exhaustion, owner refusal, uncertain
  outcome, and successful qualification. Every state names the responsible
  owner, affected profile, safe next action, and correlation/evidence reference.
- Smooth-path requirement: a user who already has LM Studio or Ollama running
  should not edit JSON, copy opaque identifiers, set environment variables, or
  invoke Make targets merely to configure and qualify a provider. Credentials
  and provider commands remain in the sovereign owner; Pantheon edits or invokes
  only an explicit bounded owner contract and presents its receipts.

## Milestone 0 — stabilize the product contract

- [x] Establish the user-journey blueprint and acceptance catalogue.
- [x] Select the bounded local-first ownership recommendations in the reflexive
  decision brief as planning input, without claiming implementation.
- [x] Record the control-plane, human-gate, and reflexive-rollout ADRs.
- [x] Define initial `BusinessReviewV1` and `ProjectProposalV1` declarative work
  modes with an accountable Executive Sponsor.
- [ ] Commit the documentation baseline and link every implementation milestone
  to its governing journey and acceptance cases.

## Milestone 1 — P0 participation truth

Owners: owner-native participation records; Moira is the first qualified owner
for governed agent participation. Pantheon is a client and projection host.

- [x] Audit existing owner contracts before admitting a new service.
- [x] Compose Moira's exact selected participation in workspace-profile,
  capability-negotiation, headless and TUI paths.
- [ ] Prove the composed path against a real restarted Moira local daemon,
  including not-found, refusal and transport failure.
- [ ] Define a Keystone participation projection law only after a second
  sovereign human/freeform participation record proves the shared fields.
- [ ] Project current and prior owner revision consistently without copying the
  semantic record into Pantheon.
- [ ] Show principal type, role/work mode, scope, delegation and authority
  references, budgets, grounding, accountable owner, policy epoch and expiry
  where the selected owner actually publishes them; keep missing facts visible.
- [ ] Prove that a role label grants no authority and a stale client cannot
  overwrite a newer transition.

Exit evidence: acceptance cases 71, 72, 86, 96 and 97.

## Milestone 2 — P1 declarative work modes

Owners: Rune for authored definitions, Dendrite for exact distributed/grounding
digests and Moira for workflow/deliberation instantiation. A missing lifecycle
owner may be selected only after the capability audit.

- [ ] Audit Rune, Dendrite and Moira against bounded register, resolve, activate,
  migrate, deprecate, retire and tombstone requirements.
- [ ] Publish only the missing contracts, with immutable definition digests and
  a named sovereign owner.
- [ ] Refuse executable content, bearer material, hidden routes and unbounded
  presentation schemas.
- [ ] Qualify J34 with `BusinessReviewV1` and `ProjectProposalV1` as the first two
  real definitions.
- [ ] Preserve exact-version instances across compatible and breaking revisions.
- [ ] Generate consistent TUI/headless catalogue and form projections without
  making Pantheon the semantic owner.

Exit evidence: acceptance cases 98–105.

## Milestone 3 — close J1 end to end

The existing Chronos-to-selected-Daimon prompt remains a qualified presentation
slice, not completion.

- [ ] Admit a Chronos task with an explicit agent-assisted execution profile.
- [ ] Configure and qualify the selected local provider through an owner-backed
  Pantheon surface; the existing checklist-only overlay does not satisfy this.
- [ ] Present provider/model health, conformance, effective tool/context budget,
  known deviations, evidence time, and the next safe action without exposing
  credentials or treating endpoint liveness as session/workflow completion.
- [ ] Instantiate the selected Moira workflow with stable task/effect links.
- [ ] Create or bind an exact Agora task room.
- [ ] Open a per-task Daimon session bound to the exact Chronos attempt.
- [ ] Admit addressed room messages through an explicit replay-safe prompt
  command; ordinary chatter never becomes a prompt.
- [ ] Link Dendrite grounding, Aegis decisions, Daimon progress, Vellum proposals,
  Agora responses, Chronos status/comments and Moira settlement.
- [ ] Recover every lost-reply cut without duplicate task, room, provider call,
  edit or message.
- [ ] Qualify the same journey through TUI and headless clients with detach,
  reattach and cursor continuation.

Exit evidence: J1 plus acceptance cases 1–30 and the owner-specific recovery
cases implicated by the selected profile.

## Milestone 4 — J33 human-gated ensemble decisions

- [ ] Compose the selected Moira deliberation and Dendrite concern-routing
  contracts.
- [ ] Retain independent first passes, correlation, dissent, material findings
  and owner dispositions.
- [ ] Publish `HumanGateRequestV1` and exact presentation/response receipts.
- [ ] Refuse relabeling, automatic disposition, timeout or agent impersonation as
  substitutes for a named-human response.
- [ ] Require fresh named-human go/no-go for final production rollout and
  invalidate it after any material decision change.

Exit evidence: J33 and acceptance cases 87–95 and 111–113.

## Milestone 5 — executive operating profiles

- [ ] Qualify J35 monthly/quarterly business review using `BusinessReviewV1`.
- [ ] Qualify J36 sponsored project proposal using `ProjectProposalV1`.
- [ ] Preserve Executive Sponsor accountability, human-advocate escalation,
  evidence lineage, dissent, funding/exposure bounds and fresh stage decisions.
- [ ] Prove that initial sponsorship is not replayable as final rollout approval.

## Deferred gate — J30 reflexive development

Do not implement self-rollout until build, rollout-decision, deployment,
observation, recovery-policy and recovery-execution owners are selected and
qualified. The first J30 candidate must be a low-risk stateless presentation
change. Moira, Aegis, any future registry, steward, observer, recovery machinery,
their schemas and their trust roots require a separate outer maintenance profile.

