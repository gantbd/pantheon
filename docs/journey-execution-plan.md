# User-journey execution plan

Status: active

Governing blueprint: [`use-cases-and-user-journeys.md`](use-cases-and-user-journeys.md)

This plan turns the product journeys into bounded delivery increments. A checked
item means the named owner contracts, refusal/uncertainty behavior, interactive
and headless projections, and executable evidence all exist. Presentation alone
does not close an item; input profiles share one behavior facade.

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
- The first owner-backed inspection slice is now qualified. Both
  `daimon-local-agent` and `daimon-local-model` validate their complete selected
  document before publishing a versioned redacted provider catalogue. Pantheon
  invokes that contract only for the explicitly hosted Daimon owner, bounds its
  runtime/output, runs inspection off the input/render loop, validates catalogue
  identity/digests/cardinality, renders
  model or agent conformance facts, and offers observable `r` revalidation
  through the central behavior facade. A real hosted-agent/tmux gate proves the
  current and refreshed projections; the no-owner breadth gate refuses fallback
  file inspection. Commands, arguments, paths, credential names and secrets
  never enter the projection.
- Daimon's interactive-agent owner now publishes a guarded provider edit
  transaction. It validates the complete current owner document, requires its
  exact expected digest, produces a redacted exact-change plan and confirmation
  digest, revalidates the complete candidate, then applies only those reviewed
  bytes under an adjacent lock with permission-preserving atomic replacement and
  parent-directory synchronization. A verified retry reports
  `already_applied`; stale state, mismatched confirmation and concurrent edits
  fail with distinct operator orientation. Commands, arguments, paths,
  environment names and credentials remain absent from the plan and receipt.
  The executable journey proves plan, redaction, apply, replay, stale-state
  refusal and the resulting owner catalogue.
- Pantheon now consumes that contract through a bounded, keyboard-driven form:
  generated provider/profile identities, direct or macOS Seatbelt isolation,
  ACP or Claude stream transport, multiline argument/environment lists, and
  optional authentication method. The form validates obvious shape errors
  before contact, performs plan and apply off the render/input loop, validates
  typed owner plan/receipt facts, requires explicit arm/confirm, fences exit and
  profile restart while the owner operation is pending, and records observable
  result instrumentation/activity. Its boundary copy explicitly distinguishes
  configuration from qualification, task binding, delegation, persona
  activation and invocation. The hosted tmux journey drives the production
  form through redacted plan and atomic apply, checks the replaced owner
  document, and cleanly drains the hosted session.
- Provider inspection no longer belongs to the terminal presentation layer.
  One internal owner-backed facade now invokes, bounds, decodes and validates
  the redacted catalogue for both TUI and aggregate headless presentation. The
  headless report publishes the same exact owner/document/provider/task/context
  facts and redaction boundary, or an actionable refusal without fallback file
  inspection; it does not expose launch or credential material. The real
  hosted Daimon/Vellum/tmux journey proves the headless catalogue before the
  guarded TUI edit and again after restart, including the changed owner digest,
  policy digest and redacted launch shape.
- This does not complete onboarding: the local-model owner now has guarded
  owner-backed edit/apply, while endpoint/model discovery, health checks, and
  tool-continuation conformance remain open. The owner now does expose the same explicit
  one-byte supervised lifecycle as the interactive-agent application, and
  Pantheon now models its socket as a distinct model-protocol endpoint rather
  than leaking it through legacy agent-presence semantics. A hosted profile
  selects exactly one model endpoint or exact agent selection; headless uses
  `connect_model`, while the Daimon tab presents authenticated attachment plus
  the explicit boundary that attachment is not execution, provider health,
  qualification, delegation or authority. The real Aegis/Chronos/Daimon gate
  hosts `daimon-local-model`, observes the same redacted Qwen/OpenAI-compatible
  catalogue through headless and TUI, and proves clean drain without contacting
  the configured provider. Pantheon can therefore present a selected running
  local-model owner, but discovery, full conformance and task invocation remain
  owner-contract gaps—not reasons to mutate raw JSON or
  call socket liveness “ready.”
- The next bounded owner slice adds completion qualification without promoting
  it to full provider readiness. Pantheon asks the selected local-model owner
  for its exact redacted document/profile, presents the endpoint scope, model,
  wire and policy, then requires review, arm and confirm before one bounded
  request. Daimon commits an operation marker before dispatch, caps retained
  history, validates the expected owner-document digest, never retries transport
  uncertainty, and retains only attributed usage, byte count, evidence digest,
  host-observed time, disposition and deviations—not model output. Replay
  returns the same receipt without another provider call. A hermetic live gate
  proves one HTTP request, TUI/headless receipt parity and clean drain; an
  explicit opt-in live-spend gate then proved the same path against LM Studio's
  loaded `qwen/qwen3-coder-next`. Both surfaces state that tool continuation
  remains `not_exercised` and that completion qualification is not task invocation,
  delegation, workflow completion or authority.
- The owner-backed configuration slice closes the raw-file escape hatch for the selected
  local-model provider. A guarded Pantheon form loads exact editable fields from
  Daimon while never projecting credential values, and remains available to
  repair a profile even when its referenced token is absent. Review, arm and
  confirm bind an atomic, replay-safe owner-document replacement; restart and a
  fresh qualification remain explicit next steps. Agent and model owners now
  share bounded read, digest, process-scoped lock and durable replacement
  mechanics instead of duplicating a crash-stale lock. The hermetic tmux gate
  proves edit/apply, a changed owner-document digest, requalification, fresh
  headless disclosure and one provider request.
- Direct dogfooding then loaded this execution plan as Pantheon's hosted Vellum
  workset. The footer advertised `Ctrl-F`, but the key reached editing behavior,
  inserted the intended query into the document and marked it dirty instead of
  opening find. Pantheon's two-step dirty-exit guard correctly prevented and
  then discarded the unsaved edit. The defect is retained in the dogfood
  Chronos owner as priority-8 task
  `vellum-ctrl-f-routes-to-edit-instead-of-find`, requiring the
  `vellum-key-routing` capability; it was not hidden as test-only feedback.
  The remediation extended Vellum-TUI's shared service input facade with a
  typed find intent and a projection-local, non-editing literal search, then
  consumed that intent through Pantheon's bounded prompt and instrumentation.
  Peer unit tests prove cursor movement without an owner effect or dirty state;
  the real `make run` tmux journey now proves `Ctrl-F`, query submission,
  visible match selection and `Service Current / Clean` before guarded exit.
  This local-model slice dogfooded the correction again by loading this exact
  plan through Pantheon's hosted Vellum, locating both the distinct
  model-protocol endpoint and its non-qualification boundary, observing the
  success feedback and clean buffer, then draining the hosted topology with
  `Ctrl-Q`.
- This qualification slice used hosted Vellum for an actual plan edit, not only
  context inspection: one exact atomic replacement advanced the document to
  revision 1 and the ordinary owner save returned it to `Service Current /
  Clean`. The attempted stronger durable-save route correctly refused because
  `make run` does not provision a stable authenticated Vellum receipt identity.
  That default-path friction is retained in the same dogfood Chronos owner as
  priority-7 task `make-run-vellum-durable-identity`, requiring capability
  `vellum-durable-receipts`. The source-save success notice also remained at
  “awaiting peer state” after the projection was already clean; final
  settlement feedback should be reconciled when that path is addressed.

### 2026-11-18 — governed local-model task invocation

- The next J1 slice now carries one newly created Chronos task through an exact
  `daimon-local-model` binding into a human-reviewed terminal-model attempt.
  Pantheon validates the current owner document, completion qualification,
  provider/profile, principal, task incarnation/attempt, context references,
  tool capabilities and complete budget before it will open the arm/confirm
  surface. The UI grants no authority and never calls LM Studio directly.
- The Daimon owner now exposes a bounded `--invoke-task` contract. It commits a
  private operation marker before contacting its already-running model service,
  derives the exact execution envelope from owner configuration, retains the
  attributed output/usage/evidence receipt, and never redispatches a replay or
  an uncertain operation. Its redacted catalogue publishes only a content-free
  latest-invocation summary, giving fresh headless observation parity without
  leaking model output.
- The hermetic depth gate now proves owner-backed provider edit, mandatory clean
  topology drain/relaunch, completion qualification, TUI Chronos task creation,
  exact binding inspection, review/arm/confirm, one successful model execution,
  replay without provider redispatch, fresh headless receipt parity and clean
  reverse drain. Exactly two provider calls occur: qualification and invocation.
- The gate exposed a stale-profile boundary leak in its previous sequence:
  qualifying an edited owner document did not update the already-running frozen
  model service. The model fixture rejected the old model name and the operation
  correctly settled `outcome_unknown`. The journey now enforces the UI-published
  restart requirement before qualification; it does not conceal the defect with
  a retry. This is a reusable rule for every owner-backed runtime configuration
  change.
- Result presentation initially placed the response and safe next action below
  the inspector viewport. Dogfood moved disposition, usage, response, deviations
  and orientation ahead of the full reviewed envelope so success and uncertainty
  are immediately observable while the exact binding remains inspectable.
- Vellum was used for the small Daimon CLI and selectable Chronos-fixture edits
  and for this plan update through guarded atomic replace/save. It still refuses
  the 520 KiB terminal source under its 64 KiB source contract with the generic
  `EditorUnavailable` result, and runtime-admitted membership cannot yet reopen
  the same persistent workspace cleanly. Chronos retains
  `vellum-large-source-chunked-projection` and
  `recover-vellum-runtime-membership-on-restart`; chunked/viewport projection
  and actionable refusal detail are required for IDE-scale dogfooding.
- The task-conversation slice again attempted to use Vellum for its own live
  gate and plan edits. Runtime admission of the gate was refused as
  `Access(OutcomeUnknown)` because the active three-document allowlist is not a
  workspace; the refusal omitted the governing root and a recovery action, then
  left the editor `Stale / Indeterminate` and fenced the admitted plan save.
  A fresh isolated owner admitted the plan and its guarded save produced a
  durable file update, but dogfood then proved that a literal match is not an
  editable replacement range and service-mode undo is unavailable. Repository
  diff review caught the resulting appended text before commit; an exact patch
  repaired it after the Vellum session was cleanly closed. Chronos now retains
  `vellum-workspace-admission-orientation`, `vellum-save-settlement-feedback`
  and `vellum-literal-match-replace-recovery` alongside
  `vellum-atomic-refusal-orientation`.
- The bounded task-to-agent-to-room relationship slice is now qualified. After
  the applied Daimon turn, Pantheon prefills a direct Agora task conversation
  between the configured operator and agent participants, retains the ordinary
  room review/arm/confirm gate, consumes the exact applied owner receipt, and
  opens a second human-gated task-conversation relationship review.
  The agent addressee is configuration, not an alias for the operator; both
  participants must already be registered with Agora.
- The addressed-message follow-on is also qualified. A command-palette action
  accepts only the explicitly selected message when it addresses the configured
  agent participant and the task-conversation owner proves one exact current
  Chronos-task-to-Agora-room relationship. It then opens the ordinary reviewed
  Daimon prompt composer; ambient room chatter remains inert. The same durable
  session accepts the second turn only after its owner again projects `Ready`
  with no active turn. `tests/live/work_item_agent_journey.sh` now proves task
  creation, first agent turn, direct conversation creation, exact relationship,
  addressed message, second agent turn and refreshed Kanban return through the
  real tmux UI.
- Depth qualification exposed three integration/presentation defects and drove
  their fixes: Agora peer cohorts require canonical lexical order at the shared
  client boundary; the prompt's safety boundary and selected body must appear
  together in the first review viewport; and post-receipt reconciliation must
  preserve compact success/boundary/next-action feedback. Vellum performed two
  exact previewed plan edits and guarded saves. A separate hosted workspace
  attempt failed the whole admission as generic `Adapter(Limit)` when one
  602 KiB root was present, without identifying that file or a recovery action;
  Chronos retains `vellum-root-limit-diagnostic` and
  `agora-agent-participant-qualification` from this run.
  Durable chat history, Chronos status/comment publication, tool continuation,
  delegated completion and approval remain separate owner contracts.
  Task-execution policy is also still raw owner configuration;
  provider/task/context/tool/budget selection must become a discoverable
  guarded UI flow before this is ordinary-user operationally complete.

### 2026-11-17 — exact agent-context receipt and Vellum presentation

- Daimon now retains a content-free receipt for the exact context admitted to
  each successful open, prompt or reattach operation. The receipt binds the
  execution, ordered context references, owner kind, source digest, byte count,
  optional Dendrite grounding evidence and a canonical manifest digest. It is
  service-owned session state rather than a synthetic provider event, so it
  cannot collide with or impersonate provider event cursors.
- The receipt survives Daimon's checkpoint/recovery and replay paths. The agent
  wire profile advanced and rejects the prior shape; the checkpoint codec
  advanced while retaining an explicit prior-version decode path. Provider
  refusal or an invalid/mismatched context assembly cannot publish a receipt.
- Moira projects the receipt into an independent public type and incorporates
  it into retained provider-evidence digests. A changed Vellum source digest
  therefore changes the evidence identity even when agent events are otherwise
  identical. Moira's agent wire frame family also advanced and rejects the old
  shape rather than reinterpreting it.
- Daimon-TUI renders the exact Vellum reference, source digest and byte count in
  the session surface. Pantheon's observation-only headless projection exposes
  the same identity and explicitly publishes `content_disclosure:
  not_published`; neither path copies source content into presentation state.
- The hosted Daimon live qualification now opens a real resumable agent session
  with one configured `vellum_document` context, drains and reopens the durable
  owner, verifies the exact receipt first through Pantheon headless JSON, then
  drives the production Ratatui surface in tmux and observes the same execution,
  reference, kind, source digest and byte count. It then continues through the
  existing guarded provider-edit journey and clean hosted drain.
- Full Daimon-TUI and Moira workspace suites passed. Daimon's full suite reached
  unrelated process-boundary timing failures only under four-way parallel
  load; the complete process-boundary test target passed serially. Pantheon's
  69 neighboring library tests and the new headless test passed in the broad
  run; the native Chronos socket test passed when rerun with its required local
  socket permission.
- This closed the retained identity/projection leak for context that Daimon had
  already assembled. The follow-on owner slice now rejects static fixture text
  labeled as Vellum evidence: `vellum_context` binds an exact public Vellum
  workspace/document/revision, resolves fresh source through the authenticated
  public SDK at open/turn time, refuses unavailable or revision-drifted owner
  evidence without fallback/cache, and retains only the generic content-free
  receipt. The redacted catalog projects runtime owner binding metadata without
  paths, credentials, source digest or byte count before execution.
- Revision `0` is valid owner state. A focused contract test accepts it, proves
  exact-byte SHA-256 identity and refuses a mismatched configured revision.
  Offline validation checks only bounded paths/schema/opaque identities and
  does not load credentials or contact Vellum.
- The hosted gate now starts authenticated Vellum, resolves the document during
  real Daimon session creation, persists and reopens the agent receipt, compares
  it with Vellum's own headless document/revision/witness/bytes/SHA-256 manifest,
  then drives the same receipt and redacted runtime binding through Ratatui.
  Daimon and Pantheon full workspace suites pass serially; the live hosted gate
  cleanly drains Vellum, Daimon, Chronos and Aegis.

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
- [ ] Give Vellum a first-class revision-bound diff/review surface for document
  and multi-document changes, with provenance, stale-coordinate mapping,
  semantic-impact evidence and accept/refuse behavior routed through the shared
  behavior facade.
- [ ] Make Vellum whole-workspace aware under an explicit resource policy:
  eagerly retain the manifest, resource metadata, declaration/reference index,
  diagnostics and revision graph; keep the active workset and bounded dependency
  closure hot; load other document bodies and parse trees on demand. Project
  index generation, scope, freshness, incompleteness, eviction and memory/context
  budgets so IDE-like resolution never masquerades as complete or current.
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
  submit path sent only task IDs and silently applied owner defaults. The first
  correction now opens a bounded pre-submit review for every exact task, exposes
  all five owner fields, submits one atomic `SubmitGraph`, preserves the reviewed
  graph when dispatch cannot be queued, and shows applied values in both hosted
  and standalone projections. A real Pantheon/tmux qualification proved
  structured capture, scheduling, independently receipted cancellation, clean
  shutdown and exact durable reopen. Rich owner/impact/evidence material remains
  deliberately outside this content-minimized record.
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
- The provider-catalog slice used Pantheon's live structured Chronos composer to
  admit `local-model-owner-composition` at priority 8 and
  `guarded-provider-edit-qualification` at priority 9, each requiring the
  `daimon-provider` capability. Chronos applied one atomic two-task graph, then a
  clean Pantheon drain and owner restart reopened revision 1 with both exact
  Ready identities. The malformed first draft produced while driving tmux was
  explicitly discarded before submission, providing useful confirmation that
  review/escape does not leak an accidental task into owner state.
- Vellum discovery dogfood used the same `make run` surface to admit one atomic
  two-task graph: `vellum-revision-bound-diff-review` at priority 7 with the
  `vellum-diff` capability and `vellum-bounded-workspace-intelligence` at
  priority 8 with `vellum-semantic`. Chronos advanced to revision 1, projected
  both as Ready, cleanly drained, and reopened the same revision and tasks in a
  fresh Pantheon process. The first item owns source-revision-bound before/after
  review, provenance and stale-coordinate behavior. The second owns a bounded
  workspace inventory/index plus an inspectable per-run context manifest with
  coverage, freshness, exclusions and resource budget; it does not mean placing
  every workspace byte in every model prompt.
- The roadmap itself is now represented in the same durable Chronos owner, not
  only in this document. Pantheon admitted a twelve-node milestone/stream graph
  at revision 2 with priorities, capability requirements and dependency edges;
  a clean owner restart reopened fourteen total tasks with five Ready and nine
  correctly Blocked. Nodes cover the discovery loop, operator agenda, contract
  traceability, participation history, observability correlation, declarative
  work modes, four J1 delivery clusters, human-gated ensembles and executive
  operating profiles. This Vellum document remains the rich acceptance source;
  Chronos owns schedulable identities and state.
- The active J1 worker/session node was then decomposed through Pantheon's live
  batch composer into seven individually executable tasks: receipt contract,
  profile validation, process controller, UI routing, receipt tests, live
  qualification and capability/documentation closure. Chronos advanced to
  revision 3 with 21 tasks, seven Ready and fourteen dependency-blocked. The
  children name required capabilities such as `rust-small-edit`, `test-fixture`,
  `process-control`, `behavior-facade` and `live-qualification`; they do not
  hard-code a model. Provider/model assignment is a separate evidence-backed
  Moira decision, so Gemma, Qwen, Codex or another provider can be selected or
  replaced without changing work meaning.
- An executable delegation packet still needs an authorized linked Vellum
  artifact: exact objective and scope, allowed files, invariants, prohibited
  boundary changes, acceptance commands, context-manifest identity, budget and
  required human review. Chronos's content-minimized task record is not enough
  input for a smaller model by itself.
- The delivery operating contract is now a linked Vellum artifact and defines
  per-turn goal reconciliation, continual TUI/headless dogfooding, friction
  remediation, stakeholder-equitable human gates, evidence-based model fit,
  overnight safety, and UI-accessible MBR/QBR reviews with an accountable
  executive sponsor. Vellum durably saved the maintenance annotation through
  Pantheon's guarded TUI path. The same pass exposed three Vellum UX gaps:
  oversized-root refusal lacks size/limit/remediation facts, large paste lacks
  an explicit partial-workset receipt, and edit backpressure can return
  `EditorUnavailable` without retry orientation. Together with the implicit
  Moira saga-endpoint dependency, these were admitted through Pantheon's live
  Chronos batch composer as four prioritized capability-tagged tasks at durable
  revision 4 (25 total tasks, 11 Ready and 14 Blocked).
- The operating-review contract now makes every meaningful turn reconciliation
  traceable source material for the next UI-visible MBR or QBR, including its
  named sponsor, open alignment questions, human gates and outcome evidence.
  While checking the live project boundary, `make status` was unavailable even
  though `make run` and the qualification targets exist. That discoverability
  gap is retained as user friction for Chronos triage; it does not weaken or
  delay the sponsored-review acceptance requirement.
- The cyclic review contract now also requires evidence-grounded inventory of
  underused capabilities, observability and presentation gaps, abstraction and
  leakage checks, and explicit learning flow through Vellum, Dendrite, Moira,
  Chronos and Hermes. Hermes remains an uninterpreting ordered domain-event
  ledger; Keystone receives only repeated, semantically identical mechanisms
  with independent consumers and cross-consumer qualification, never Pantheon
  policy or owner semantics.
- Engram and Pies are now standing prior-art inputs under a fractal review that
  moves across breadth/depth, up/down, in/out and lifecycle time. The linked
  transfer map distinguishes direct reuse, owner-preserving adaptation,
  presentation inspiration and rejection; each accepted lesson must name its
  Pantheon journey, sovereign owner, invariants, qualification and learning
  flow through Dendrite, Moira, Chronos and Vellum.
- The meta-goal now names an epistemic center for each material review: identify
  the smallest load-bearing truth, adversarially drive decidable `maybe` claims
  down to scoped Boolean evidence, and escalate genuinely value-laden,
  plural/under-specified questions to explicit mu and named-human judgment.
  Operational uncertainty remains an owner fact and cannot be relabeled mu.
- Rune now supplies the executable discipline for that review rather than only
  prior-art prose. Real `scry`, `machine_path_check`, `smith` and Dendrite MCP
  runs validated the audit, gap-analysis, ensemble, instrumented-BDD and UADD
  structures, loss-checked the audit-cycle Mermaid projection, and retrieved
  exact user-experience guidance. Pantheon's canonical delivery-governance flow
  validates and projects to Mermaid/RDF/SHACL while keeping the unselected
  review-definition owner visibly unresolved. Dogfood also exposed two retained
  ecosystem gaps: local Dendrite startup orientation around required secrets,
  and legacy groundings that refuse modern bounded surface depths and require
  exact `describe` fallback. Structural proof remains distinct from semantic
  truth, workflow execution and named-human acceptance.

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

The dogfood path now also exposes four guarded one-turn collaboration lenses—
Executive Sponsor, Product Owner, Project Manager and IC Collaborator—through
the same command/behavior facade and Daimon prompt review used for ordinary
work. The prepared prompt explicitly says that a lens is not human identity,
organizational title, work-mode activation, delegation, authority, approval or
sign-off. This makes role-shaped collaboration usable now and testable in the
task-to-agent journey, while intentionally leaving durable role activation,
provider selection and authority binding to their sovereign owner contracts.

The guarded interactive-agent provider transaction is the first configuration
primitive needed to close that gap. The remaining Pantheon slice must compose
it with provider selection and qualification rather than equating a successful
configuration write with a delegated turn. An operator still reviews the exact
task, provider/profile, persona lens, grounding depth, tools, budget and human
gates before invocation; the resulting task, session, room and owner receipts
must share durable correlation identities so the operator can return to
Chronos and see evidence-backed status or comment proposals.

The selected execution profile no longer fabricates a fresh task attempt for
each control. It now requires one explicit task key/incarnation/attempt and
retains it across prompt, permission and reattach while keeping execution and
idempotency identities fresh per exact request. The Configuration inspector and
the same workspace profile's headless observation project the same task,
principal, harness, context/tool counts and budgets; every absent
role/delegation/accountability/
authority fact remains visibly `not_published`. The work-item and three
session-set tmux gates enforce the exact envelope in the Daimon fixture. This is
P0 relationship truth, not J1 completion. The follow-on owner-binding gate now
limits the composed journey to one exact Chronos task ID, waits for its applied
receipt, asynchronously inspects Daimon's validated owner catalogue, and
requires that task's published external key to equal the frozen execution key.
Missing, ambiguous, wrong-owner or mismatched evidence stops before prompt
preparation and tells the operator that the task exists but no agent was
invoked. The next qualified slice now continues that exact binding through a
human-gated fresh-session review. A validated workspace profile supplies the
opaque workspace and bounded session budget; Pantheon generates fresh session,
execution and idempotency identities, reviews the exact owner envelope, and
dispatches once only after the operator arms it. The owner-created session is
then admitted into an initially empty Daimon presentation workset, observed as
one exact Ready session, and only then receives the carried work-item prompt.
Outcome-unknown fences that session identity for inspection without prompt or
redispatch. `tests/live/daimon_open.sh` proves the shared standalone/Pantheon
headless guard, while `tests/live/work_item_agent_journey.sh` drives the real
tmux surface from Chronos task creation through session open, prompt acceptance,
and return to the Kanban board. Headless observation projects the open policy
without inventing a preexisting session.

Pantheon can now also present one already-admitted Moira agent participation and
ask Moira to prepare its exact Open effect through a review/arm/confirm sheet.
The public client sends only the participation ID; Moira validates pristine
Admitted state and derives the revision, idempotency key and execution identity
from its complete retained binding. A real SQLite/Unix qualification proves one
durable preparation, exact replay after daemon restart and no presentation-side
identity invention. This is an owner-bound effect-preparation seam, not yet the
J1 chain: participation admission/template instantiation, worker dispatch,
per-task Daimon session creation and automatic Chronos-to-Moira routing remain.

The next worker/session sub-slice now has an implementation boundary. An
optional workspace-profile field selects the `moira-agent-local` executable and
opaque owner configuration. After the exact Open is durably prepared, a
separate guarded workbench action launches that one-step owner process off the
UI thread. Moira's process receipt now publishes the exact participation ID,
outcome and retained revision; Pantheon bounds output, refuses a mismatched
participation, keeps failure/uncertainty visible and refreshes owner state after
a definitive result. The owner repository's real-process test crosses Moira and
Daimon sockets and commits Ready observation; Pantheon's receipt/profile tests
are green. The composed qualification now observes Admitted through headless
mode, drives Open preparation and the real owner worker through the TUI,
crosses Moira and Daimon sockets, observes the exact revision-3 receipt in the
activity surface, and proves Ready through both a second headless observation
and direct durable reopen. The live pass also exposed and fixed a real serial-
owner deadlock: the Open review now deterministically releases its authenticated
owner session before the separate worker connects, then Pantheon reconnects a
fresh participation observer. `agent.worker.step.v1` is therefore qualified.

The earlier observation-only profile copy was a presentation-boundary leak.
The aggregate headless observer now validates requirements against the complete
configured workspace route, exercises only its actual observation catalogue,
and emits every other valid profile requirement as `not_exercised`. Unknown
requirements still refuse. This makes one profile observable across adapters
without claiming that session-set control ran or that control parity is closed;
bounded session-set machine commands with owner challenges, per-child receipts,
uncertainty stops and explicit unsent-child recovery remain required.

On 2026-11-17 the implementation review itself was delegated to the running LM
Studio `qwen/qwen3-coder-next` model through Daimon's bounded OpenAI-compatible
provider gate. Daimon verified nonempty attributed output, provider-reported
input/output usage, the frozen provider-document digest and the evidence digest.
The accountable integrator accepted the “boundary leak plus missing behavior”
diagnosis but rejected a redundant access-type layer (Pantheon already publishes
capability planes), configuration-based control whitelisting, a synthetic
`--attest` authority flag, and a generic SQLite receipt queue. The run therefore
proved useful reviewed contribution, not autonomous approval or delivery.

It also exposed a material dogfood gap: Pantheon can configure/drive interactive
ACP sessions, while its governed terminal-model path is not yet operationally
reachable from a task. The developer had to enter through Daimon's qualification
seam, and the model received a curated prompt rather than bounded repository
exploration, iterative challenge, or a typed proposal schema.

Vellum is now part of the dogfood context path rather than a ceremonial output
target. The same assessment document was opened and edited through the live
Pantheon TUI and observed through hosted headless Pantheon as exact document,
revision, witness, byte-count and SHA-256 facts; headless output deliberately
withholds source text. A tmux-driven raw edit containing `]` exposed global
surface navigation stealing focused editor input. The behavior facade now gives
printable brackets to focused Vellum while retaining direct Ctrl/Alt, palette and
mouse navigation, and the exact failure case passes live. This established
content identity and presentation parity evidence. The follow-on owner slice
now establishes agent-context parity as well: Daimon consumes the configured
Vellum reference through the authenticated public SDK and its durable receipt
matches Vellum headless's exact source digest. The run also exposed that a
persistent TUI connection currently excludes a second headless Vellum client;
attach/concurrency remains an owner-level gap rather than a reason to fork
workspace state.

The longer dogfood session then exposed two presentation/runtime gaps. Idle
polling exhausted Vellum's bounded 256-request replay register in about a
minute, and Pantheon had not enabled terminal bracketed-paste mode, so a
multiline atomic replacement was interpreted as source and command keystrokes.
Vellum clients now renew only after unresolved effects are absent, rebuild the
new identity's exact inventory/view/selection/snapshot witness, and preserve
local persistence truth only when the owner snapshot is equal. Pantheon checks
capacity at each editor action as well as idle polling and records renewal;
bracketed multiline paste is bounded, line-ending normalized and all-or-none.
Real tmux gates cross the old request ceiling and execute a three-document
multiline atomic replacement through independent durable receipts.

The same Vellum workset delivered Moira's first content-free execution-cohort
correlation slice. The governed harness now retains shared principal, profile
and context-manifest facts; `moira-methodical-local` projects them to operators.
Shared principal/profile identity is `correlated`, while distinct identities
remain `not_established` rather than becoming an independence claim. Provider,
model, organizational and training-lineage correlation remain explicit gaps.

- [x] Admit one exact Chronos task and prove its owner-published binding to the
  already selected agent-assisted execution profile before prompt preparation.
  Dynamic profile selection and attempt creation remain open.
- [ ] From the selected task, choose a qualified local provider, declared work
  mode/persona, bounded delegation, grounding depth, tools and budgets through a
  reviewable Pantheon form; show the effective owner-backed configuration before
  invocation.
- [ ] Configure and qualify the selected local provider through an owner-backed
  Pantheon surface. Completion qualification now produces a replay-safe,
  owner-retained receipt through the guarded TUI and shared headless catalogue,
  owner-backed edit/apply now loads exact non-secret fields and produces a
  replay-safe atomic receipt, but endpoint/model discovery and tool-continuation
  conformance are not exercised.
- [ ] Present provider/model health, conformance, effective tool/context budget,
  known deviations, evidence time, and the next safe action without exposing
  credentials or treating endpoint liveness as session/workflow completion.
- [ ] Instantiate the selected Moira workflow with stable task/effect links.
- [x] Create or bind an exact Agora task room through explicit human gates. The
  separately owned `agora-chronos-adapter` publishes a durable
  `task-conversation.v1` local owner/client with exact replay/conflict,
  bidirectional lookup and explicit broken-link projection. Pantheon's shared
  behavior facade now reviews, arms and confirms the exact selected Chronos task
  and open Agora room, reports the owner receipt, and projects the same
  content-free relationship status headlessly. `tests/live/task_conversation.sh`
  and `tests/live/work_item_agent_journey.sh` qualify complementary real-TUI
  paths, headless parity and owner restart recovery. The composed work-item
  journey now includes the Agora capability: after an applied Daimon turn it
  prefills a direct conversation for the configured operator and agent
  participants, retains Agora's ordinary review/arm/confirm gate, and opens the
  separately gated exact relationship from the applied owner receipt.
  Standalone room creation and manual binding remain independently available
  capabilities.
- [x] Open a per-task Daimon session bound to the exact Chronos attempt.
- [x] Expose a governed terminal-model invocation from the selected task so a
  configured LM Studio/Ollama provider can contribute without a test harness or
  raw HTTP; retain exact task/execution/profile/context/tool/budget identities,
  usage, evidence, refusal and uncertainty recovery. The first qualified slice
  is one reviewed terminal attempt; Agora conversation, Chronos update and
  tool-continuation composition remain separate unchecked items below.
- [ ] Qualify coding delegation with bounded read-only repository exploration,
  explicit context lineage, iterative reviewer challenge and a typed proposal;
  keep integration, authority and acceptance with the accountable human/agent
  ensemble rather than the provider response.
- [ ] Allow an explicit participation transition between dogfood modes without
  silently changing principal identity, authority, accountability, tool grants,
  budgets or approval rights. Each switch retains the prior activation,
  rationale, scope and owner revision.
- [x] Admit addressed room messages through an explicit replay-safe prompt
  command; ordinary chatter never becomes a prompt. The shared behavior facade
  requires one selected addressed message, one exact current owner relationship
  and one owner-projected ready Daimon session, then routes the bounded evidence
  through the ordinary prompt review/arm/confirm flow. The live J1 gate proves a
  second accepted turn and user-visible reconciliation feedback.
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

### Tellus identity and dogfood findings — 2026-11-18

- [x] Make Tellus the single component identity across Pantheon surfaces,
  behavior commands, profiles, capability manifests, headless reports, Dendrite
  routes, fixtures, source qualification and live gates. `grounding` remains a
  domain term for Rune inputs/digests only; there is no legacy component alias.
- [x] Qualify Dendrite `/tellus/v1/observe` → Tellus typed client → Pantheon
  headless and tmux TUI, including partial-peer failure without loss of healthy
  Tellus evidence.
- [x] Dogfood Vellum through Pantheon: admit a second project document with
  Ctrl-O and retain `vellum-workspace-limit-diagnostics` as a real Chronos task.
- [ ] Replace Vellum's opaque `Workspace(Session(Adapter(Limit))` startup
  refusal with the violated bound, observed value, configurable owner setting,
  and safe next action. The current whole-file seed rejected `terminal.rs`.
- [ ] Select and qualify a bounded whole-workspace/context-index profile for
  Vellum. Progressive file admission works, but it is not IDE-like project
  context or cross-file symbol resolution.
- [x] Correct the dual-service dogfood journey to use semantic behavior-facade
  routes from Vellum (`M-1`/`M-2`/`M-3`); bracket keys remain document input.
  The gate now deliberately proves one edit and dirty-exit protection rather
  than relying on an accidental navigation-key edit.

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

The first stakeholder-truth dogfood and categorical no-go assessment is
retained in
[`ensemble-dogfood-assessment-2026-11-17.md`](ensemble-dogfood-assessment-2026-11-17.md).
It proves the bounded owner/runtime foundation while refusing to call a
single-provider, partly modeled, selected-message-only run an operationally
mature stakeholder ensemble.

- [ ] Compose the selected Moira deliberation and Dendrite concern-routing
  contracts.
- [ ] Make stakeholder standing first-class and distinct from persona: retain
  constituency/domain, representation kind, accountable principal, mandate and
  conflicts for every member; label modeled perspectives and technical SMEs so
  neither can impersonate an affected stakeholder.
- [ ] Ask and retain “Do we have the right ensemble?” before substantive review.
  Project `yes`, `no` or `contested` with affected-population, equity/coverage,
  missing/excluded/overrepresented and assessor evidence; refuse substantive
  disposition on `no` and escalate rather than average away `contested`.
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

