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

### 2026-11-18 — provider posture at the J1 human gate

- The exact task-to-model binding previously retained Daimon's qualification
  receipt but reduced it to an operation ID before showing the invocation
  review. The review now carries the owner-published completion disposition,
  observation time, evidence digest, tool-continuation posture and known
  deviations alongside the effective context/tool budget. Runtime health is
  explicitly `not published`; endpoint liveness is not restyled as readiness or
  task success. The screen gives one bounded next safe action and separately
  labels proposal review, preserving the human gate.
- The hosted local-model gate now requires those facts on the real tmux surface
  before invocation and retains owner observation time in the completion
  receipt. This advances the provider-health checklist without closing it:
  continuous runtime/service health and endpoint/model discovery remain owner
  contract gaps. The first dogfood pass incorrectly assigned runtime health to
  Daimon. The ownership audit caught that leak: Pantheon cancelled
  `daimon-local-model-runtime-health-projection` at board revision 28 and
  admitted priority-8 `hestia-runtime-readiness-daimon-correlation` at revision
  29. Hestia's first private-process slice now publishes bounded loopback
  TCP-accept readiness basis, observation time and opaque endpoint identity,
  while retaining launch details and process handles inside Hestia.
- Hestia commit `12c4a5c` advances that owner boundary into a standalone local
  daemon and ordinary bounded client. The daemon retains lifecycle state and
  OS process handles across fresh client connections, refuses shutdown while a
  runtime is active, and makes an exact retry after a discarded start reply
  resolve to the retained ready incarnation rather than spawning a duplicate.
  Private configuration selects executable, arguments, environment and
  readiness policy; none crosses the public protocol. Strict format, Clippy,
  all-target workspace tests, rustdoc and a process-separated daemon fixture
  qualify the slice. Persistence across abrupt daemon loss, orphan
  reconciliation, artifact/executable digest verification, enforced resource
  and isolation posture, and the cross-owner Pantheon presentation remain open.
- The same dogfood pass used Pantheon's hosted Vellum to edit Hestia's README.
  `C-s` visibly found the selected phrase, but typing inserted after it instead
  of replacing the active selection. The receipt was honest and the edit was
  repaired without concealing the failure. Priority-8 Chronos work
  `vellum-find-selection-edit-semantics` was admitted through the live TUI at
  owner revision 30; selection replacement and an orienting success/failure
  presentation remain required before that path is editor-faithful.
- Pantheon now consumes Hestia's ordinary client through an explicit one-to-one
  provider/runtime binding. Headless observation and the guarded local-model
  invocation review preserve Hestia authority, runtime, revision, incarnation,
  lifecycle, readiness basis/time, endpoint reference, model identity, resource
  expectations, control posture and fence state beside—never merged into—
  Daimon's qualification and execution evidence. A configured runtime that is
  missing, mismatched, fenced or not Ready refuses before the invocation review
  can arm. The real tmux gate starts the provider under Hestia, observes it in
  headless mode, drains and restarts it as incarnation 2 after a provider
  change, shows the correlated evidence at the human gate, invokes through
  Daimon, and cleanly drains both owners. The gate also exposed and corrected a
  fixture crash on Hestia's probe-only TCP connection, reinforcing that
  TCP-accept readiness is not provider conformance.
- Hestia commit `ece6dba` now also ships a bounded ordinary `hestiactl` client for observe,
  start, drain and clean shutdown. It emits one versioned JSON result and reads
  bounded absolute request documents; lifecycle decisions remain daemon-owned.
- Hestia commit `8a58abc` adds the reusable `hestia-tui` Ratatui component.
  Pantheon now gives Hestia a dedicated tab and routes its keys and command-
  palette actions through one semantic behavior path. Exact Ready evidence,
  retained intent, review, arm, one-shot drain/restart, settled refusal and
  outcome-unknown guidance stay owner-native. The real tmux journey used Alt-I
  to operate incarnation 1→2, then replaced the provider model and observed
  incarnation 3 at the Daimon invocation gate and through headless parity.
  Restart persistence/orphan reconciliation, stronger artifact/resource/control
  enforcement and Aegis authority remain open.
- Hestia commit `903c98b` adds the first separately authorized model-artifact
  catalog and acquisition owner slice. It discloses the exact source revision,
  license, format, expected size and SHA-256; streams through a private bounded
  adapter into an owner-private partial file; verifies before atomic admission;
  and keeps acquisition distinct from instantiation, qualification and use.
  Pantheon now supports artifact-only and mixed Hestia profiles, switches its
  Hestia tab between runtime and artifact panes, routes keys and minibuffer
  commands through the same semantic facade, and reports owner evidence
  headlessly. The real tmux qualification exercises the human gate, admits a
  23-byte fixture, and proves that payload bytes never enter the repository.
  Live registry search, signatures, compatibility/resource impact, restart
  reconciliation, cancellation, multi-file manifests, retirement and the
  artifact-to-runtime bridge remain open. Devstral is the first intended real
  dogfood artifact after an explicit storage/license/resource decision.
  OpenRouter is a later Daimon provider-integration backlog item rather than a
  Hestia lifecycle shortcut.
- The next Hestia slice closes the single-file artifact-to-runtime bridge.
  One hosting profile binds an exact catalog artifact to a private server
  argument position. Hestia refuses missing/unadmitted artifacts and complete
  model-identity mismatches, then rechecks regular-file type, expected size and
  SHA-256 immediately before starting the owned process. The composed live gate
  proves admission → owner-private path injection → Hestia readiness → Daimon
  qualification/invocation → clean drain, and deliberately proves that a
  staged/final model mismatch leaves the initial headless report incomplete.
  Qwen is still served by LM Studio: the fixture qualifies the ownership
  contract, not a production Qwen deployment. Real GGUF/engine qualification,
  multi-file MLX/Safetensors manifests, observable large-file verification and
  resource/isolation enforcement remain open.
- The opt-in real-model gate now pins Qwen2.5-Coder-1.5B-Instruct Q4_K_M at
  Hugging Face revision `2ab9f8f42af02fc212effaef7c4850c885e965f4`, exact
  size `1,117,320,768` and SHA-256
  `cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046`.
  Hestia acquired and admitted those bytes, reverified them at start, injected
  the private path into its configured `llama-server`, served a real completion,
  and drained and shut down cleanly. This exposed and fixed a presentation/client
  boundary leak: the owner allowed a 300-second startup, while `hestiactl` and
  `hestia-tui` abandoned lifecycle replies after five seconds. Lifecycle workers
  now retain a bounded 420-second settlement window while observations retain
  their short deadline. It also empirically proved that Hestia TCP acceptance
  can precede provider readiness (`/v1/models` returned 503 while loading), so
  the gate checks those facts separately. The composed variant now presents the
  same exact Hestia runtime in Pantheon, qualifies its live endpoint through
  Daimon, performs one human-gated invocation for a Chronos task, proves
  headless provider/runtime model correlation and invocation eligibility, and
  drains both owner topologies cleanly. Persisted admission/restart and
  hashing/loading progress remain open.
  Qwen3-Coder-Next continues to be served by LM Studio.
- Live Pantheon/Chronos dogfood admitted the two independently executable
  follow-ups at owner revision 37: priority-8
  `hestia-real-model-daimon-composition` and priority-6
  `hestia-model-start-progress-observability`. The batch form preserved the
  exact reviewed graph across invalid field input and submitted both changes in
  one owner revision. Existing `hestia-sovereign-local-inference-host` work was
  reused rather than duplicated.
- The Qwen2.5-Coder 1.5B response from the composed run was retained but rejected
  as architecture guidance: it repeated generic requirements and invented
  encryption and a 24-hour review target absent from the evidence packet. The
  owner/harness path was correct and fully attributed. Categorization:
  model-capability and role-fit failure, not context transport, authority,
  orchestration or lifecycle failure. This supports using the small model for
  bounded mechanical verification while retaining stronger models and an
  accountable integrator for architecture decisions.
- The opt-in Hestia specialist-model smoke bench now pins, acquires, verifies,
  serves and drains three complementary Apache-2.0 GGUFs in addition to the
  Qwen coding baseline: SmolLM3 3B Q4_K_M at revision
  `4965cb60b150737b68a0408c36aeefb65078f894` (1,915,305,312 bytes, SHA-256
  `8334b850b7bd46238c16b0c550df2138f0889bf433809008cc17a8b05761863e`),
  OLMo 2 1B Instruct Q4_K_M at revision
  `62f8c199538474c3e33ed5d7e0580abd66686a27` (935,515,296 bytes, SHA-256
  `abd8187934a438fbf7cfff0a1de5b9d2793ce913f158794df1951dcba6c93cc6`),
  and Granite 4 H-Tiny Q4_K_M at revision
  `08d5a8a9741dd5c1a95d2d39e25253226aa1464e` (4,230,976,352 bytes, SHA-256
  `5a38b08c441ae1adbafb1d2b8a7167e0d48734d83af68b268cefea1eec553dcd`).
  OLMo correctly classified exit-zero without an expected-output assertion as
  uncertain and is the strongest immediate candidate for bounded asynchronous
  test triage. SmolLM produced a structured checklist but overclaimed generic
  completion requirements; Granite presented value and risk but placed human
  review after automatic closure. The latter two are useful contributors, not
  decision owners. Model role assignment therefore remains evidence-based and
  configurable; the smoke gate proves serving, not fitness or approval.
- The same bench exposed two Hestia product gaps under real restart conditions:
  a killed owner can leave a stale Unix socket, and an admitted artifact cannot
  yet be reconstructed from its verified on-disk digest after daemon restart.
  The safe qualification workaround uses the retained file only as a new
  acquisition source so Hestia recounts and rehashes every byte into a fresh
  store; normal users must not have to recopy multi-gigabyte models. Durable
  catalog restoration, stale-owner recovery and observable cold-start phases
  remain P0 usability work.
- LM Studio remains deliberately installed as the third-party provider testing
  ground so attach/federate behavior cannot collapse into Hestia-specific
  assumptions. Ollama currently retains Gemma3 27B and two Qwen3 variants
  (about 52 GB total) and may be retired only after Hestia plus Daimon/Pantheon
  prove equivalent required coding/specialist invocation, readiness and
  operator-observation behavior. Removal is an explicit post-parity action, not
  an inferred consequence of one successful local completion.
- Live Pantheon/Chronos dogfood admitted priority-8
  `hestia-artifact-catalog-restart-recovery` and priority-4
  `ollama-retirement-after-hestia-provider-parity` at revision 39, followed by
  priority-7 `oculus-local-model-lifecycle-telemetry`, priority-6
  `local-model-specialist-ensemble-evaluation`, and priority-4
  `acp-harness-provider-privacy-qualification` at revision 40. The ensemble
  item means evaluation through Moira's deliberation/saga ownership: Dendrite
  supplies grounded concern routing, Daimon invokes exact participants, Hestia
  or LM Studio serves models, Agora carries attributed conversation, Oculus
  observes, and Chronos tracks the work. None of those peers independently
  acquires Moira's cohort, synthesis, dissent, uncertainty or disposition
  authority. Claude, Codex, Gemini, Muse and future ACP harnesses require
  explicit credential, context-disclosure, retention and egress qualification;
  provider availability is not permission to disclose workspace context.
- Two bounded Qwen3 Coder Next reviews completed through Pantheon's
  Daimon/Chronos gate but were not accepted as approval evidence. The capacity
  proposal contradicted its own byte arithmetic and invented a Vellum protocol;
  the narrower Hestia review reported three blockers contradicted by the code
  (pre-hash model comparison, missing index validation and an acquisition/start
  race despite serialized owner dispatch). The harness delivered and retained
  both results correctly. Categorization: model/context-grounding and verifier
  calibration failure, not a transport or orchestration failure. Qwen remains
  useful for decomposed candidate work, with deterministic checks and an
  accountable integrator retaining acceptance authority.
- Moira's heterogeneous live-provider qualifier now invokes four distinct
  provider/model profiles against one exact question and a provenance-bearing,
  read-only Vellum projection. The model runs have zero tool budget and no
  ambient shell/filesystem access; Dendrite supplies the grounded method and
  concern cohort, Daimon invokes, and Moira retains attributed member responses
  without resolving them. The first Hestia small-model cohort admitted only one
  of four reports: SmolLM3 was structurally valid but contradictory, OLMo omitted
  required provenance fields, Granite appended invalid transport punctuation
  and alleged absent checks that were present, and Qwen2.5 fenced its JSON and
  supplied a generic conclusion. Raw evidence SHA-256 is
  `fae3e15432b1b54921fd0197ad30fda174908af128f4db2e4d8fe2f8ba4c2a0a`.
  This is a role-fit and output-profile no-go, not a serving or authority
  failure; those models remain candidates only for narrower qualified work.
- A second cohort used LM Studio as the retained third-party provider ground:
  Qwen3 Coder Next, GPT-OSS 20B, Qwen3.6 35B-A3B and GPT-OSS 120B produced four
  distinct, valid, independently attributed reports in 161.86 seconds. Raw
  evidence SHA-256 is
  `c4544a30f1a5997212407899a602724b3711d5a63f3e7ca1c0579056b3909ff9`.
  The cohort is operationally useful but remains a no-go as decision authority:
  every report included at least one overreach or policy-as-defect inference,
  the configured cohort does not yet prove affected-stakeholder equity, and no
  human disposition was supplied. Moira correctly left the review unresolved.
  The next slice must preserve independent reports in Agora, add grounded
  rebuttal/falsification and owner dispositions, and answer “Do we have the
  right ensemble?” with stakeholder/standing evidence rather than static test
  configuration.
- Live Chronos dogfood captured the associated friction/gaps without duplicating
  existing work: `moira-ensemble-capability-discoverability`,
  `hestia-artifact-recovery-progress-telemetry`,
  `hestia-parallel-runtime-start-orchestration`,
  `vellum-client-attach-readiness-retry-contract`,
  `tellus-runtime-capability-version-discovery`,
  `secret-safe-peer-discovery-configuration`,
  `chronos-batch-mode-transition-input-fencing`,
  `daimon-provider-uncertain-diagnostic-receipt`,
  `vellum-bounded-symbol-context-projection`, and
  `daimon-structured-output-profile-negotiation`. The board reached revision 46
  with 61 tasks. Tellus also exposed that the long-running Dendrite binary
  predates its current Tellus route: MCP grounding works, while
  `tellus.observe.v1` currently falls through with 500 until the owner is safely
  replaced or capability negotiation refuses the stale instance.
- The next J33 slice now has an executable Moira review-exchange foundation.
  It admits review only after every independent first pass, retains exact
  findings and evidence, requires a different admitted member for rebuttal,
  blocks owner disposition until challenge exists, preserves all seven finding
  disposition families, binds policy-gated findings to the exact admitted
  named-human response, and recovers an exact checkpoint without deriving
  authority from Agora speech. The Moira–Agora adapter projects those retained
  references only into explicitly addressed messages. Focused core,
  canonical-codec, public-wire, SQLite restart/fencing and full Agora-adapter
  qualifications are green.
  Moira now retains the exchange in the same revisioned deliberation registry,
  encodes it canonically, migrates legacy registries without inventing review
  history, and restores reports, findings and rebuttals through the fenced
  SQLite owner after restart. Typed review refusals also round-trip over the
  public wire. The public capability-shaped mutation/read clients now carry
  begin-review, finding, rebuttal, disposition and exact review observation
  through the canonical bounded wire and service. A SQLite close/reopen journey
  proves that the same reports, findings, rebuttals and dispositions survive
  and remain readable through that public boundary. Named-human approval
  updates the retained review, exact lost-reply replay returns the same review
  outcome, and a gated disposition cannot proceed without the admitted
  approval. A real Agora `ServiceV2` round trip now starts from one retained
  review checkpoint, projects the exact finding, rebuttal and disposition,
  publishes three independently receipted addressed messages, and observes the
  same bodies and recipients from Agora's owner. That journey exposed and fixed
  canonical recipient ordering; duplicate recipients now refuse before
  dispatch. A review-specific recovery journey then discards the first real
  Agora success reply, recreates the durable publisher and proves exact request
  replay returns the retained receipt without a second room message.
  Review creation now also requires one complete correlation disclosure over
  execution principal/profile, context manifest, model, provider, prompt,
  grounding, evidence scope and organization plus first-pass exposure. Moira
  retains it in the review checkpoint, derives only `correlated` or
  `not_established`, publishes the same evidence through protocol profile v4,
  and restores it through SQLite. Missing dimensions remain zero-observation
  disclosures; distinct values never become an independence claim. Legacy v1
  reviews migrate as explicitly undisclosed rather than receiving invented
  evidence. The reusable Moira presentation now reads deliberation and review
  under one stable registry-revision bracket. Pantheon headless emits the exact
  disclosure and its tmux TUI shows first-pass counts plus every observed or
  unknown dimension. A real SQLite-backed owner qualifies both surfaces with
  observed execution-principal/grounding evidence and unknown provider evidence
  while retaining `not_established`. Moira commit `4c0980a` then closed the
  text-only publication boundary: one adapter-owned typed correlation binds the
  exact deliberation, finding/rebuttal/disposition and effect to every accepted
  or refused Agora outcome. Dispatch is available only through the shared
  `publish_review` facade; the raw transport intent is inspectable only through
  bounded body/address views. Real Agora and SQLite qualifications prove stale
  refusal, lost-reply recovery, independent store reopen and exact replay
  without a second room message. Authoritative producers for the still-unknown
  correlation dimensions remain required before the J33 checklist item closes.
- Chronos revision 53 atomically admitted three follow-on dogfood discoveries:
  priority-6 `chronos-complete-paginated-task-search`, priority-5
  `pantheon-canonical-dogfood-workspace-attachment`, and priority-4
  `pantheon-last-known-good-development-launch`. They own complete bounded-board
  search and total/page-cap disclosure, unambiguous reattachment to the active
  roadmap instead of a fresh default state root, and an explicit qualified
  binary path when unrelated peer source is temporarily uncompilable.
- The default `make run` owner separately admitted priority-8
  `moira-reviewer-correlation-evidence` at revision 9 and reopened it exactly as
  Ready with its Moira-deliberation, Moira–Daimon and Pantheon-presentation
  capabilities. Its separation from the revision-53 roadmap is further direct
  evidence for `pantheon-canonical-dogfood-workspace-attachment`, not a second
  roadmap authority.
- The same owner reached revision 10 after a TUI backlog-reconciliation pass
  admitted priority-5 `vellum-search-repeat-and-incremental-isearch` with
  Vellum-editor, Keystone-keybinding and Pantheon-presentation capabilities.
  This is the independently executable owner gap behind the observed missing
  repeat-next-match behavior and the documented non-incremental `C-s` search.
  The pass deliberately did not duplicate the still-open correlation producers
  or review-item publication linkage: both remain acceptance work on the
  reopened `moira-reviewer-correlation-evidence` task.
- A source-to-owner audit then decomposed the remaining producer work at
  revision 11 into priority-7
  `daimon-model-invocation-correlation-evidence`, with Daimon-model,
  Moira–Daimon and Moira-deliberation capabilities. Moira currently receives
  execution principal/profile, requested context references and grounding, but
  Daimon's public model result omits the provider profile/document, model
  identity and assembled message/prompt evidence that its service actually
  used. Those values must be projected from Daimon's retained invocation rather
  than inferred from output or local configuration. Organization stays
  explicitly unknown until a named identity/organization owner supplies that
  relationship; neither provider nor principal identity is a valid substitute.
- Daimon commit `7e2757b` now publishes that content-free invocation evidence
  from the exact retained invocation through protocol/wire v4: provider profile
  and document digest, model digest, ordered context-manifest digest,
  message/prompt-manifest digest, and an order-insensitive evidence-scope
  digest. Legacy results remain explicitly unavailable rather than receiving
  invented evidence, and checkpoint restoration recomputes the projection from
  its retained invocation. Moira commit `ef331a7` consumes the typed evidence,
  validates exact outcome membership, and derives observed context, model,
  provider, prompt and evidence-scope dimensions without inferring
  independence. Organization remains explicitly unknown. Focused protocol,
  wire, service, checkpoint and Moira adapter tests plus strict Clippy passed.
  Moira commit `0241140` now composes that analyzer into the governed local run
  after exact member outcomes return, while preserving cohort refusal before
  plan admission. The full local-harness package, strict Clippy and a Pantheon
  headless two-document Vellum composition passed. Moira commit `ada8469` then
  binds that exact governed checkpoint and correlation into explicit live
  review creation. A real Dendrite-grounded, Daimon-executed four-member run
  reopened the SQLite review with all four model/provider/prompt/evidence-scope
  observations retained. Stale checkpoints refuse, review and resolution cannot
  share one run, active review suppresses ready-to-resolve, and tool-continuation
  review refuses until multi-turn evidence aggregation exists. Organization
  and that multi-turn evidence remain acceptance work on
  `moira-reviewer-correlation-evidence`. Moira commit `76295c5` additionally
  derives first-pass exposure from harness-owned sequencing only after the
  complete no-tool cohort is durably committed: the live review reopens with
  four committed-before-cross-review members, zero early exposures and zero
  unknowns. Partial and continuation runs retain unknown exposure and cannot use
  this path.
- Pantheon TUI dogfood then admitted priority-8
  `chronos-asynchronous-verification-worker` at Chronos revision 12, blocked on
  `j1-worker-process-controller` and `j1-worker-receipt-contract`, with explicit
  Chronos-worker, Daimon-test-runner, Oculus-observability and
  local-model-triage capabilities. This preserves the economical boundary:
  Pantheon executes allowlisted deterministic tests and owns receipts; a local
  model may select bounded slices, triage structured results and propose task
  updates; Chronos owns lifecycle/evidence, Oculus owns ELMT, and humans retain
  waivers and rollout authority. The backlog contract is now executable and
  discoverable, but unattended task-to-runner-to-status operation is not yet
  delivered.
- Vellum dogfooding admitted the exact Moira–Agora source file but rejected a
  three-file workspace at its configured limit. More importantly, it retained
  a stale clean snapshot after an external source mutation and offered no
  behavior-facade refresh action or conflict explanation. Chronos revision 50
  now retains `vellum-external-change-refresh-conflict`; the existing bounded
  symbol-context item retains the multi-file capacity evidence.
- Live Pantheon/Chronos dogfood admitted four owner-visible Ready items at board
  revision 6: priority-9 `hestia-model-artifact-acquisition-devstral`, priority-2
  `daimon-openrouter-provider-integration`, priority-5
  `agora-supervisor-local-agent-operating-topology`, and priority-5
  `oculus-persistent-sre-and-delivery-agents`. This keeps immediate local model
  acquisition, later hosted-provider portability, supervisory delegation and
  persistent operational/delivery roles independently schedulable.
- The same admission exposed severe synthetic/bulk-input latency: tmux literal
  task IDs were consumed approximately one key per render/poll cycle, leaving a
  valid atomic batch visibly half-entered for tens of seconds. The graph did
  eventually settle exactly, but this is poor human and agent ergonomics.
  Priority-6 `pantheon-tui-bulk-input-throughput` was admitted at board revision
  7; bounded paste/batch handling and progress feedback must share the behavior
  facade rather than introducing an automation-only mutation path.
- Keystone is directionally appropriate for a small semantic-neutral input
  mechanism—bounded chords, prefix state, profiles and conflict detection—but
  not for component commands, behavior facades or Ratatui state. Live Pantheon
  dogfood admitted priority-4 Ready work
  `keystone-keybinding-mechanism-extraction` as Chronos incarnation 25. This is
  deliberately behind operational Hestia work and requires at least two real
  TUI consumers before any general Keystone qualification claim.

### 2026-11-18 — owner-work admission and contribution delivery

- Live Pantheon/Chronos dogfood admitted `chronos-external-work-admission-contract`
  and `daimon-contribution-delivery-contract` at owner revision 15.
- Chronos public coordination can submit and cancel work, while claim and
  completion remain a trusted in-process worker profile with native grants and
  leases. J1 must therefore enter prospectively through Moira's trusted
  receiver: admit the work, claim and renew it with private Chronos authority,
  invoke Daimon, retain the exact result in the owning Moira workflow, then
  settle or fence Chronos from that retained evidence. The receiver now exposes
  this in-process admitted-result bridge and the durable four-store path proves
  it without adding a wire or bearer completion API. Normal-user owner-worker
  packaging and Chronos status/comment projection remain open; historical
  out-of-band work remains honestly Ready rather than adopted after the fact.
- Daimon task invocation retains task, execution, provider, context, usage,
  disposition and output-digest evidence and returns bounded safe text to the
  authenticated caller. Pantheon already presents a 4,096-character MODEL
  RESPONSE preview in the guarded workflow. The real gap is durable delivery
  into Agora plus Chronos comment/status proposals; raw provider HTTP is not an
  acceptable hidden collaboration path.
- The explicit LM Studio gate now qualifies Qwen3 Coder Next, creates the exact
  Chronos task through Pantheon, reviews and confirms one Daimon invocation,
  presents the response, verifies content-free headless parity and retains exact
  usage/digest evidence. The reviewed run used 132 input and 1,504 output tokens
  for 6,642 safe-text bytes with zero effect proposals.
- The Qwen review correctly preserved non-bearer authority and named-human
  review, but invented a Solidity EvidenceAdmitter, on-chain mappings,
  addresses, IPFS, oracles and task/grant/lease APIs absent from the evidence
  packet. The accountable integrator rejected the proposal. Categorization:
  model/training-inertia overreach amplified by thin domain grounding; the
  Pantheon/Daimon harness delivered and attributed the requested work correctly.
- The same run exposed a presentation-state leak: invoking Create a Chronos task
  from the Execution roster dispatched the behavior but left the roster
  topmost, making the task form invisible and input inert. Chronos-TUI now
  dismisses presentation-only overlays before entering guarded task and cancel
  workflows. Unit, strict Clippy and hosted tmux restart gates qualify the fix.
- A later Vellum/Chronos dogfood pass reproduced two user-facing inconsistencies:
  the compact header advertised `Ctrl-2` for Vellum but Overview did not route
  it while the behavior-facade command palette succeeded, and an automated
  overlay-close/save burst returned to an already-Clean view without proving the
  newer document generation reached disk. A separated retry then refused the
  now-larger plan as `SnapshotLimit` with no partial file change. Chronos revision
  17 retains direct-chord parity; the existing Vellum workspace/large-source and
  save-settlement items retain the bounded projection and generation-specific
  feedback work.
- Chronos commit `ca12fe2` now exposes one bounded, non-cloneable in-process
  worker handle from its writable atomic owner. Ordinary coordination admission
  and native claim/complete share one serial durable envelope; no grant, lease,
  request, or result is serialized. Full owner tests caught and drove fixes for
  a duplicated poll descriptor and clean worker-channel EOF. Restart after a
  committed claim projects Uncertain and refuses a second claim. Moira still
  owns that behavior in separate `moira-chronos-worker` commits `233408c` and
  `727660c`: native grants and leases remain private, exact effect/execution/
  task/attempt evidence is required, and unsupported dispositions refuse before
  an owner call. Its isolated real-owner profile proves completion, mismatch,
  uncertainty without reclaim and configuration refusal. Pantheon's native host
  now returns the sole Moira-wrapped capability from the same Chronos owner and
  proves it fences after clean drain. The task-to-session runtime still needs to
  retain and use that capability; the live revision-18 task therefore remains
  Ready rather than being manually relabeled complete.
- The live LM Studio gate now accepts a bounded review-context file and used the
  real Pantheon task-to-Daimon path for an independent Qwen critique. Qwen
  correctly prioritized reply-loss, restart, drop and persistence-fence tests,
  but incorrectly treated nonblocking `try_recv` as blocking, reversed the
  send-before-wake ordering, conflated Chronos state mutation with an external
  effect and proposed two-phase reply delivery. Categorization: useful grounded
  review with distributed-systems training inertia; harness attribution and
  retained evidence were correct, and the accountable integrator accepted only
  the source-grounded test recommendations.

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
  The agent addressee is configuration, not an alias for the operator. Agora's
  owner now accepts a bounded schema-2 create-time cohort, and the hosted live
  gate bootstraps distinct human and agent participants. Runtime registration
  and a discoverable owner-backed Pantheon configuration form remain open.
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
- The owner-cohort slice exposed a pre-existing lifecycle boundary leak while
  qualifying schema 2. Pantheon's interactive shell retained Agora's live
  projection client while asking the single-connection private-local owner to
  drain; the owner could not admit its shutdown wake, Pantheon killed it at the
  deadline, and the exact socket inode remained as truthful crash residue.
  Pantheon now releases every Agora presentation client before owner drain.
  The unchanged schema-1 baseline and the new schema-2 human/agent cohort both
  pass the full hosted TUI, clean shutdown, refused shutdown, abrupt loss,
  outcome-unknown and exact residue-recovery gate.
- Vellum admitted and rendered this plan in an isolated Pantheon session. An
  exact query targeted text that a repository patch had already removed while
  the live owner refreshed underneath the session. Vellum-TUI incorrectly
  collapsed the resulting zero-match preparation into generic `InputLimit`,
  which prompted an initially wrong workset-limit diagnosis. The presentation
  facade now distinguishes invalid query, zero matches and genuine owner/service
  refusal; the real tmux gate proves the complete recovery message before one
  successful three-document atomic edit and durable save. This paragraph was
  itself corrected and saved through Pantheon's guarded Vellum path. The
  misclassified Chronos item was cancelled rather than retained as false work.

### 2026-11-18 — hosted local-inference lifecycle

- Local-provider onboarding now distinguishes an attached endpoint from a
  Hestia-owned inference runtime. Attachment observes an independently owned
  endpoint; Hestia is a sovereign peer accountable for admitted placement and
  process lifecycle, including the future distributed target. Pantheon composes
  Hestia's public client rather than owning model-process handles. Daimon
  continues to own provider configuration, qualification, invocation, retained
  model evidence and recovery. Neither endpoint liveness nor Hestia ownership
  grants execution, tool, workflow or rollout authority.
- The first host profile is a bounded managed-process contract, not a model
  engine embedded in Pantheon or Hestia. Hestia binds a public opaque hosting
  profile and a private deployment plan containing exact executable identity, redacted
  arguments, credential-name references, private endpoint, model/artifact
  identity and digest where available, quantization/context facts, memory/
  accelerator/thread/concurrency budgets, sandbox and egress posture, readiness
  probe, graceful drain and orphan-recovery policy. LM Studio may remain an
  attached GUI-owned endpoint; llama.cpp, Ollama, MLX or another explicitly
  selected executable can implement the same qualified host profile without
  becoming Pantheon or Daimon semantics.
- Model acquisition is a separate human-gated operation because it can spend
  network, disk and license/trust budget. Hosting never implicitly downloads,
  updates, deletes, changes a model, kills an unknown process, reuses an
  unverified stale endpoint or retries an uncertain start. Configuration change
  requires a reviewed restart and fresh Daimon qualification before selection.
- Configuration must present Daimon attach versus Hestia host, runtime/model provenance,
  effective resource budgets and preflight results. Operations must present
  starting, loading, ready, degraded, draining, stopped, crashed, orphaned and
  uncertain states with exact Hestia owner, host/site, runtime incarnation,
  correlation, elapsed time and next safe action. Daimon presents only the
  independently qualified provider/session view. Moira can coordinate staged
  start/qualify/select/drain workflows; Chronos owns durable work; Oculus
  queries authorized telemetry sources while Hermes retains only domain and
  integration events. No default signal contains model text or credentials.
- Acceptance includes TUI/headless parity and real-process gates for clean
  start/drain, restart after configuration, path-budget preflight, stale socket,
  port conflict, missing executable/model, load failure, resource refusal,
  process crash, refused drain, orphan recovery and partial topology rollback.
  A sandbox-policy dogfood refusal reported only `socket bind failed`; hosted
  inference must name the violated path/port/resource/policy bound and safe
  recovery instead of repeating that opacity.
- Pantheon's live Chronos owner admitted priority-8 task
  `pantheon-local-model-hosting` at revision 18. The initial delivery target is
  the lifecycle/profile/receipt boundary above plus one hermetic managed-process
  adapter; broad runtime support and automatic model management are not implied.
- The same live owner then admitted priority-8 task
  `chronos-live-owner-headless-attach-readiness` at revision 19 after a second
  headless Pantheon client reported `not_ready`/`not_sent` against the visibly
  live interactive owner. The TUI creation path composed, reviewed and applied
  the item exactly. Attach readiness therefore remains a separate parity and
  diagnostics defect; it is not folded into hosted-inference lifecycle semantics.
- The operator then selected the sovereign-peer boundary. An abandoned internal
  Pantheon launch-profile draft was removed before commit. The same live owner
  admitted priority-9 `hestia-sovereign-local-inference-host` at revision 20,
  priority-8 `oculus-observability-query-peer` at revision 21, and priority-8
  `tellus-operational-presentation-breadth` at revision 22. Hestia receives the
  placement/lifecycle protocol and private process adapter; Oculus receives the
  source/query/follow/pivot contract over external telemetry backends; Tellus
  must become an operational provenance/query surface with headless parity.
- The Tellus breadth audit found that the existing tab and standalone client
  honestly expose only `tellus.observe.v1`: load identity, state, freshness and
  digest. Rune/method discovery, SME roster, provenance chains, bounded query,
  source explanation and deep links remain absent owner contracts, not merely
  Pantheon rendering work. The obvious palette phrase `open tellus` also found
  no action even though `navigate.tellus` exists; the footer did not orient the
  operator to Tellus's `Ctrl-6`/`Alt-6` route.
- Vellum dogfood used a fresh owner-backed session to update `README.md`: one
  exact match at revision 0, owner generation 2 atomic publication, then guarded
  save back to Clean. A stale session separately turned tmux `F6` into literal
  `[17~` bytes and service mode refused undo; the bytes were removed without
  saving over the externally changed journey plan. The live Chronos owner
  admitted priority-6 `pantheon-palette-navigation-aliases` at revision 23 and
  priority-7 `vellum-function-key-escape-and-undo` at revision 24.
- The bounded palette defect was then repaired centrally: command kinds publish
  natural search verbs while execution still returns the existing command ID to
  the behavior facade. A focused test proves `open tellus` resolves only
  `navigate.tellus`; a rebuilt tmux-driven TUI run selected that one action and
  opened the Tellus landing with the ordinary success notice. No second
  navigation implementation or surface-specific special case was added.
- The live owner admitted priority-9 `pantheon-emacs-keymap-profile` at revision
  25. The first implementation adds a validated `pantheon`/`emacs` workspace
  choice, Configuration disclosure, shell-restart diff, and stateful Emacs
  `M-x`, `C-g`, `C-x C-f`, `C-x C-s`, `C-x C-c`, `C-h b/?` and `C-h l`
  chords. They resolve to existing semantic commands; `C-p` and `C-q` are no
  longer stolen in Emacs mode. The documented acceptance boundary keeps
  Vellum-native motion, mark/region, kill/yank, buffer and undo work open.
- The next dogfood slice removed remaining Pantheon-keymap leakage from that
  profile. Raw `C-o`, `C-s`, Ctrl/Alt-number and bracket navigation no longer
  silently retain Pantheon meanings. `C-x b` opens the existing minibuffer
  prefiltered to component surfaces; `C-x Left/Right` moves between them; and
  `C-a/e/b/f/n/p/d` plus `C-s` normalize into Vellum's existing cursor, delete
  and bounded-find actions without implementing editor semantics in Pantheon.
  A real hosted Chronos/Vellum tmux gate exercises those chords, `C-x C-f`,
  `C-x C-s`, and guarded `C-x C-c`, and is part of `make tui-demo`.
- Cross-shell dogfood confirmed that current attachment is explicit
  profile/endpoint attachment, not ambient process discovery or Pantheon-session
  federation. A second shell can connect to independently owned services when
  given their exact selected endpoints and authority, but it cannot inherit
  another Pantheon process's hosted children. The live board already carries
  priority-8 `pantheon-running-peer-discovery-attach`; it remains the right home
  for a bounded authorized catalogue/handoff/status experience, so no duplicate
  task was created. A missing `make status` target is additional operator
  friction, not permission to add process scanning.

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

Qualified 2026-11-18 first-source slice: Moira's exact pending
`HumanGateRequestV1` now enters the shared operator agenda as a blocking item
with the owner request identity, registry revision, named reviewer, reason,
consequence of waiting and a route to deliberation. A settled response remains
deliberation evidence and is not left on the agenda. TUI and headless projections
share that rule, and agenda presence explicitly grants no authority. This does
not close the multi-owner agenda or provide an action control: Pantheon still
needs an Aegis-authorized, command-scoped Moira response path with stale/reply
recovery before a human can settle the gate from this surface.

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

Pantheon's governed terminal-model path is now operationally reachable from an
exact Chronos task: the live Qwen qualification used Pantheon's reviewed Daimon
invocation, retained attributed usage and digest evidence, and presented bounded
safe output. Invocation is no longer the material gap. Bounded workspace
discovery and context, independent ensemble challenge, a typed proposal schema,
durable Agora delivery, and Moira-mediated Chronos status/comment settlement
remain open. The model still received a curated prompt rather than repository
exploration or iterative challenge, which contributed to the rejected overreach.

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
correlation slice. The governed harness observes principal, profile,
context-manifest and grounding facts; `moira-methodical-local` projects them to
operators. The review owner now retains those facts alongside explicit unknown
model, provider, prompt, evidence-scope and organization dimensions and
first-pass exposure. Shared observed values are `correlated`, while distinct
values remain `not_established` rather than becoming an independence claim.
Authoritative producers for the still-unknown dimensions remain open. The
shared Moira TUI/JSON projection and Pantheon headless/tmux composition now
present the same retained disclosure; missing evidence is no longer hidden.

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
  The invocation gate now presents every owner-published qualification,
  deviation, evidence-time and effective-budget fact and explicitly reports
  runtime health as unpublished. This remains open until the provider owner
  publishes a bounded current runtime-health projection rather than liveness.
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
- [x] Bootstrap a bounded, distinct human/agent Agora cohort through the
  sovereign private-local owner's versioned create configuration. Schema 1
  remains unchanged; schema 2 accepts 1–16 unique participants, requires the
  authenticated principal in the cohort, and preserves participant kinds. The
  full Pantheon-hosted lifecycle gate proves the cohort without importing
  runtime registration authority. Existing-store participant administration
  and the normal-user Pantheon configuration journey remain open.
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
- [x] Make stakeholder standing first-class and distinct from persona in the
  owner protocol and Pantheon read-only presentation: retain
  constituency/domain, representation kind, accountable principal, mandate and
  conflicts for every member; label modeled perspectives and technical SMEs so
  neither can impersonate an affected stakeholder.
- [ ] Ask and retain “Do we have the right ensemble?” before substantive review.
  Project `yes`, `no` or `contested` with affected-population, equity/coverage,
  missing/excluded/overrepresented and assessor evidence; refuse substantive
  disposition on `no` and escalate rather than average away `contested`.
- [ ] Retain independent first passes, correlation, dissent, material findings
  and owner dispositions. Moira now has a replay-safe, registry-revisioned and
  SQLite-restored review exchange for first-pass admission, cross-member
  rebuttal, findings, seven explicit owner dispositions and exact human-gate
  response binding, plus public capability-shaped review mutation/read commands
  and an addressed Agora projection. Review creation now retains a mandatory,
  explicit nine-dimension correlation disclosure and first-pass exposure;
  public profile-v4 and SQLite-reopen qualifications prove that shared evidence
  stays `correlated`, distinct evidence stays `not_established`, and missing
  evidence stays visible. Real `ServiceV2` publication and discarded-reply
  recovery are qualified. Typed deliberation/item/effect correlation now
  accompanies accepted, refused and replayed publication outcomes through one
  durable adapter facade. Daimon now authoritatively publishes and Moira
  consumes model, provider, prompt and evidence-scope evidence from the exact
  retained invocation. The governed local run now returns those observed facts
  and explicit live review creation retains them through SQLite reopen. Exact
  first-pass exposure is also retained for a complete no-tool cohort from
  harness-owned sequencing. Authoritative organization evidence and correlation
  across tool-continuation invocations remain open.
- [ ] Make an Agora room a durable shared-context venue for an equitable
  stakeholder ensemble, not merely a transcript or a fan-out prompt. Qualify
  the concrete operations dispute in which an SRE agent links recurring-issue
  evidence and its Chronos work, a project-management agent proposes
  deprioritization, and either participant can convene the right stakeholders
  to make the case. Preserve participant standing, independent positions,
  evidence links, dissent, addressed turns and the exact task/decision scope;
  room speech alone must not change Chronos priority. Route the resulting
  recommendation or unresolved conflict through Moira to the named owner or
  human gate, then project the attributable disposition back into both the room
  and linked work without claiming consensus that did not occur. The existing
  `agora_shared_agent_context` live gate is fixture evidence for the opening
  exchange only; it does not yet prove durable multi-turn deliberation,
  adjudication, or owner-applied priority disposition.
- [x] Publish `HumanGateRequestV1` and exact presentation/response receipts.
- [x] Publish owner-resolvable, revision-bound references for the complete human
  decision packet and response rationale. The packet must let the reviewer see
  the exact decision, alternatives, evidence, dissent, uncertainty,
  reversibility and deadline while retaining the existing binding digests;
  hashing unretained text or asking a human to approve opaque digests is not
  acceptable evidence. Prefer a selected Vellum/Hestia owner adapter over
  importing document semantics into Moira.
- [x] Compose the selected decision-packet reader through Pantheon in headless
  and interactive modes. A trusted workspace-profile route maps one opaque
  Moira owner/artifact identity to one exact Vellum workspace/document, the
  reader continuously verifies revision, SHA-256 and size, and the Moira pane
  presents the human-readable packet or an actionable refusal without gaining
  edit or response authority.
- [ ] Compose a separately selected, command-scoped human-response control with
  review/arm/confirm, four distinct dispositions, Aegis authorization, exact
  success/refusal evidence and lost-reply recovery. Do not expose the control
  until the complete current decision packet resolves and verifies.

Qualified 2026-11-18 material-reader slice: `moira-vellum` now reuses its
existing opaque external-document route to select one exact Vellum document,
read the public snapshot, enforce a bounded packet size and verify both revision
and SHA-256 before returning human-readable text. Unknown routes, invalid route
scope, stale revision, changed bytes, refusal and transport failure remain
distinct; the reader carries no edit or decision authority. Moira commit
`874e9b4` passed the complete workspace suite. At that point this qualified the
resolution behavior but did not complete the checklist above:
`HumanGateRequestV1` still lacked the opaque packet reference, and
`HumanGateResponseV1` still lacked a resolvable rationale artifact reference.

Qualified 2026-11-18 reference-contract slice: Moira now requires one opaque
owner/artifact route, expected revision and SHA-256 for the decision packet and
for the attributable response rationale while retaining the existing binding
digests. Zero routes, zero revisions and zero digests refuse at the domain
boundary. The public protocol, bounded wire codec, service mapping, durable
checkpoint and `moira-vellum` reader carry the same reference without importing
Vellum document semantics or granting decision authority. Deliberation
checkpoint bytes advance explicitly to v5; legacy v3 without gate state
migrates, while v4 is refused rather than reinterpreted. `moira-tui` exposes the
references in headless JSON and the reusable pane. Pantheon's pending agenda
initially advanced from `blocked` for missing references to `inspect_only`. The
selected reader is now composed through the workspace profile: headless
observation and the interactive Moira pane reuse the same `moira-vellum`
adapter semantics, expose verified human-readable text, and distinguish an
unresolved or stale packet from current evidence. The item remains
`inspect_only` because the separately authorized response control is still
absent. `tests/live/moira_deliberation.sh` now has Pantheon host Vellum, author
and save the packet through the TUI, discover its exact opaque
workspace/document and evidence headlessly, admit that evidence through the
real Moira fixture, then prove packet text and agenda parity in headless and
tmux-driven interactive modes.
- [ ] Refuse relabeling, automatic disposition, timeout or agent impersonation as
  substitutes for a named-human response.
- [ ] Require fresh named-human go/no-go for final production rollout and
  invalidate it after any material decision change.

Qualified 2026-11-18 inspection slice: Moira now owns an exact
`HumanGateRequestV1` and response contract binding the named reviewer, finding,
decision, material revision and policy. The response distinguishes approved,
rejected, deferred and changes-requested, injects verified Aegis authority, and
recovers an exact lost success without accepting a conflicting stale response.
`moira-tui` owns one reusable exact TUI/headless projection for admission,
stakeholder standing and pending/settled gate evidence. Pantheon can attach only
the independent deliberation endpoint, select its own Moira pane through the
behavior facade, display settled evidence in both modes, and route only a pending
gate into the operator agenda. The real-daemon gate is
`tests/live/moira_deliberation.sh`. It does not close Dendrite concern routing,
independent member execution, finding disposition, or the still-missing
Pantheon control for an Aegis-authorized named-human response. The subsequent
control audit originally found that the request's four binding digests had no
owner-resolvable artifact references and the response retained only a rationale
digest. The reference-contract slice above closes that schema gap. Pantheon now
classifies the pending entry as `inspect_only` and names the still-missing
selected reader and response control instead of presenting a misleading
approval affordance.

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

