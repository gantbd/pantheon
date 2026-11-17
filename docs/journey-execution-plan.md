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
8. Anything expected to vary by user, workspace or deployment must have a
   discoverable, validated owner-backed configuration path in Pantheon; source
   edits, environment variables and opaque profile files are escape hatches, not
   the normal user journey.
9. Let users capture discoveries into Chronos with the current evidence already
   attached. Agent-assisted sifting may recommend deduplication, ordering and
   rationale, but only an admitted owner/human transition may discard, commit or
   materially reprioritize work.

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

### Operational discovery loop

- [ ] Add a consistent `Capture in Chronos` behavior to failures, anomalies,
  missing-capability notices and qualification results. Prefill the responsible
  owner, surface, correlation identity, observed impact, reproduction/evidence
  references and a proposed acceptance check; never copy credentials or private
  content implicitly.
- [ ] Make configurable policies discoverable at the point of use and in the
  Configuration surface, including provider profiles, observation sources,
  correlation fields, time windows, cursor/freshness thresholds, metric
  cardinality budgets, sampling visibility and agent-sifting profiles.
- [ ] Retain each discovered item through Chronos's stable identity, revision,
  receipt and recovery contract rather than a Pantheon-only checklist.
- [ ] Offer a bounded agent-assisted backlog sift that proposes duplicate
  clusters, dependencies, urgency/value/risk ordering, missing acceptance
  evidence and rationale. Preserve the original items and mark uncertainty,
  disagreement and unavailable context.
- [ ] Present every proposed merge, close or material priority change for human
  review when a reasonable user advocate could contest lost scope, affected-user
  impact or sequencing; agent ranking is advice, not commitment authority.
- [ ] Qualify capture, sift, accept/refuse and restart recovery in TUI and
  headless modes, including partial batch outcomes and an unavailable agent.

### Chronos operator agenda

- [ ] Project one operator agenda across owner-native approvals, sign-offs,
  permission decisions, exception reviews, uncertain outcomes and stale or
  expiring human gates; retain each source owner's identity and revision.
- [ ] For every agenda entry show why this operator is being asked, the admitted
  authority/delegation (or its absence), requested decision, changed facts,
  deadline/expiry, consequence of acting or waiting, evidence, dissent and exact
  next owner action. Presence on the agenda grants no authority.
- [ ] Make views, notification thresholds, grouping, quiet periods, escalation
  policy and time/risk ordering configurable without hiding overdue, safety or
  human-advocate items.
- [ ] Let agents summarize, cluster and recommend order while preserving raw
  owner facts and uncertainty; only the named human or governing owner can make
  an approval, sign-off or policy decision.
- [ ] Execute accept, refuse, request-changes, delegate, defer and inspect actions
  through stable owner commands with confirmation proportional to risk,
  observable receipts and lost-reply recovery. Batch action must stop or split
  when authority, semantics or confirmation challenges differ.
- [ ] Qualify accessibility, keyboard/mouse operation, restart, expiry during
  review, stale agenda entries, replaced decisions and unavailable owners.

Exit evidence: J33/J35/J36 human gates plus acceptance cases 73, 82, 93, 111,
112 and 113; an agenda is a projection and action router, never an authority
owner.

### 2026-11-17 — Chronos backlog dogfood

- Pantheon's real writable Chronos surface atomically admitted six discoveries
  from this qualification run. The owner advanced to revision 1, projected six
  Ready items and reported `Applied. Observed 6 tasks on this page.` After a
  clean topology drain and owner restart, the same revision and all six Ready
  items reopened. A seventh local-agent delegation/persona-transition item then
  advanced the owner to revision 2 and reported `Applied. Observed 7 tasks on
  this page.`
- Captured work covers Moira-seeded signal correlation, durable role-activation
  history/no-authority proof, configurable discovery capture and agent sifting,
  the operator agenda, UI-configurable local providers, and evidence-prefilled
  `Capture in Chronos` behavior. The seventh item covers selecting Chronos work
  for a qualified local agent with an explicit persona/work-mode transition.
- The batch form made six-title capture fast, but every task landed at priority
  zero and the form could not attach owner, impact, evidence/correlation,
  acceptance checks, dependencies or proposed priority. Those omissions make
  later sifting reconstruct context and are therefore product gaps, not merely
  richer task-description wishes.
- Source audit found that Chronos already owns priority, dependencies, required
  capabilities, maximum attempts and retry-safety policy, while the reusable TUI
  submit path sends only task IDs and silently applies owner defaults. Those
  existing fields should become a validated advanced capture/configuration path.
  Chronos intentionally content-minimizes its coordination record; descriptions,
  evidence and acceptance material belong in an authorized linked Vellum
  artifact (or another selected content owner), with Moira coordinating the two
  receipts rather than inflating or duplicating the Chronos record.
- No agent-assisted sift is exposed from the board, and no configurable profile
  explains its criteria, model/provider, context boundary or decision limits.
- The first launch found a stale Vellum socket and returned only
  `binding refused: \"service\"`; a fresh root recovered. Startup feedback should
  name the conflicting endpoint/owner and offer inspect, reconnect, replace or
  choose-another-path actions rather than requiring the user to infer a stale
  socket from a generic binding failure.

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
- [x] Prove the composed path against a real restarted Moira local daemon,
  including not-found, refusal and transport failure. The isolated
  `qualification/moira-participation` gate crosses Moira's durable SQLite owner,
  public Unix client, Pantheon headless projection, and Pantheon TUI rendering.
- [ ] Define a Keystone participation projection law only after a second
  sovereign human/freeform participation record proves the shared fields.
- [x] Project the current and previously observed owner revision consistently
  without copying the semantic record into Pantheon. Moira's reusable client
  preserves only the prior revision number, refuses regressing or in-place
  rewritten projections, and retains the last trusted view with orienting
  feedback; headless honestly reports that no revision history was published.
- [ ] Recover the exact prior activation after process/client restart from its
  sovereign owner (acceptance case 96). Moira currently publishes only the
  current agent participation, so Pantheon must not manufacture durable history
  from display state.
- [x] Show principal type, role/work mode, scope, delegation and authority
  references, budgets, grounding, accountable owner, policy epoch and expiry
  where the selected owner actually publishes them; keep missing facts visible.
  Pantheon headless and Moira's reusable TUI now expose every published binding,
  method and grounding reference while explicitly identifying role/work mode,
  scope, delegation, budgets, authority reference, policy epoch, expiry and an
  accountable human as not published by this owner.
- [x] Prove against Moira's durable owner that a stale client cannot overwrite a
  newer participation transition; the restarted qualification attempts the
  stale mutation, receives `RevisionConflict`, and reopens unchanged revision 2.
- [ ] Prove that a role label grants no authority once a sovereign role/work-mode
  activation owner exists. Moira publishes a persona lens, not a role label, so
  treating that lens as acceptance-case-72 evidence would overclaim.

Exit evidence: acceptance cases 71, 72, 86, 96 and 97.

### Cross-cutting observation uplift seeded by P0

- [ ] Define a bounded correlation projection from Moira participation
  observations using the owner, participation, workflow, deliberation, member,
  session, task/attempt, revision, event cursor and grounding identities already
  qualified here.
- [ ] Let event, log, metric and trace surfaces pivot on those exact identities
  without collapsing their meanings: an owner observation is not itself a
  domain/integration event, log record, metric sample, span or alert.
- [ ] Preserve source/observed/ingest time, cursor gaps, sampling, reset,
  cardinality and missing-data status so a correlated view cannot manufacture a
  causal order or a falsely complete green state.
- [ ] Give users one orienting path from a participation anomaly to its
  correlated signals and back, with unavailable sources and the next safe action
  stated explicitly instead of requiring manual identifier copying.
- [ ] Qualify correlation through owner restart, cursor continuation, delayed
  arrival and partial telemetry failure while leaving the underlying domain
  result unchanged.

Exit evidence: acceptance cases 54–70, initially anchored by the qualified P0
Moira participation fixture rather than a parallel Pantheon-owned truth store.

## Milestone 2 — P1 declarative work modes

Owners: Rune for authored definitions, Dendrite for exact distributed/grounding
digests and Moira for workflow/deliberation instantiation. A missing lifecycle
owner may be selected only after the capability audit.

- [x] Audit Rune, Dendrite and Moira against bounded register, resolve, activate,
  migrate, deprecate, retire and tombstone requirements. The evidence-backed
  [`work-mode-owner-audit.md`](work-mode-owner-audit.md) finds complementary
  authoring, grounding and instance ownership, but no generic durable definition
  lifecycle owner; Pantheon remains a projection/action host.
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
- [ ] From the selected task, choose a qualified local provider, declared work
  mode/persona, bounded delegation, grounding depth, tools and budgets through a
  reviewable Pantheon form; show the effective owner-backed configuration before
  invocation.
- [ ] Configure and qualify the selected local provider through an owner-backed
  Pantheon surface; the existing checklist-only overlay does not satisfy this.
- [ ] Present provider/model health, conformance, effective tool/context budget,
  known deviations, evidence time, and the next safe action without exposing
  credentials or treating endpoint liveness as session/workflow completion.
- [ ] Instantiate the selected Moira workflow with stable task/effect links.
- [ ] Create or bind an exact Agora task room.
- [ ] Open a per-task Daimon session bound to the exact Chronos attempt.
- [ ] Allow an explicit participation transition between dogfood modes without
  silently changing principal identity, authority, accountability, tool grants,
  budgets or approval rights. Each switch retains the prior activation,
  rationale, scope and owner revision.
- [ ] Admit addressed room messages through an explicit replay-safe prompt
  command; ordinary chatter never becomes a prompt.
- [ ] Link Dendrite grounding, Aegis decisions, Daimon progress, Vellum proposals,
  Agora responses, Chronos status/comments and Moira settlement.
- [ ] Let the local agent propose Chronos status, priority, dependency, comment
  and follow-up changes with evidence and rationale; apply only the exact
  accepted owner transitions and keep refusal/partial/uncertain outcomes visible.
- [ ] Recover every lost-reply cut without duplicate task, room, provider call,
  edit or message.
- [ ] Qualify the same journey through TUI and headless clients with detach,
  reattach and cursor continuation.

Exit evidence: J1 plus acceptance cases 1–30 and the owner-specific recovery
cases implicated by the selected profile.

### J1 dogfood participation matrix

| Mode | Useful delegated work | Required human boundary |
|---|---|---|
| Executive sponsor | Frame outcomes, sponsorship constraints, funding/exposure bounds and stage questions | A model cannot become the accountable executive, approve funding or issue rollout go/no-go |
| Product owner | Refine user value, affected-user impact, acceptance evidence and backlog ordering proposals | Ranking is advisory; contested scope, priority and discard/merge decisions remain human/owner transitions |
| Project manager | Surface dependencies, milestones, risks, blockers, agenda items and follow-up proposals | Scheduling does not grant authority; escalation, commitment and sign-off retain named owners |
| IC collaborator | Investigate, design, implement, test, review and explain bounded work with cited evidence | Tool/file/effect scope remains explicit; authored output cannot self-review or self-approve |

The first local-Qwen dogfood should run the same bounded audit once per selected
mode, retain distinct reports and correlation/model/profile evidence, then ask a
human to compare utility and boundary adherence. Persona text alone is not a
work-mode activation and must never be used to infer delegation or authority.

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

