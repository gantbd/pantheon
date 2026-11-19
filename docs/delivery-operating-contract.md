<!-- 2026-11-18: human gates now carry packet/rationale references; pending control is inspect-only. Large governing-doc edits bypassed Vellum's current size limit. -->
<!-- 2026-11-18: hosted Vellum now settles save feedback; the 87,417-byte journey plan exceeds its current workspace limit. -->
<!-- Active delivery edits use hosted Vellum when practical; bypasses are disclosed per turn. -->
<!-- Maintained through Pantheon Vellum dogfood; see the per-turn reconciliation contract. -->

# Delivery operating contract

## Governing intent

Pantheon delivery is complete only when documented operational journeys work
through real owner processes. Polished tabs, passing unit tests, and adjacent
peer evidence do not substitute for the complete user loop: work creation,
governed delegation, agent execution, collaboration, status and comment return,
operator approval, recovery, and durable reopen.

Lessons learned may strengthen or clarify this contract, its acceptance
evidence, and the execution method. They must not narrow promised journeys,
weaken human governance, conceal an ownership boundary, or redefine success
around an easier implementation.

## Per-turn reconciliation

After every meaningful delivery turn:

1. Record what became authoritatively true and the evidence that proves it.
2. Record assumptions disproved, surprises, user friction, and boundary leaks.
3. Record how Pantheon itself was used through TUI and headless paths, plus any
   bypass and why it was necessary.
4. Decide whether each lesson requires immediate remediation, an owner-bound
   Chronos item, a human gate, or an executive decision.
5. Update the next executable slice without weakening or deviating from the
   governing journeys.
6. Announce completed work, remaining dependencies, surprises, friction, and
   the next meaningful action.

## Verification economics and asynchronous assurance

Testing depth follows change risk rather than habit. Iteration runs exact
affected tests; contract or persistence changes add the owning codec, reopen,
migration and immediate-consumer checks; strict lint covers touched workspaces
before commit. A broad suite is reserved for genuinely cross-cutting
integration or release checkpoints. Every delivery report names what ran, why
it was sufficient, and what intentionally did not run.

A built revision may hand off asynchronously to a persistent verifier so the
builder can continue without weakening the quality gate. Chronos binds the
verification lease and result to the exact commit plus dirty-state digest;
Vellum supplies the bounded diff/context; Daimon enforces allow-listed
commands, isolation and resource budgets; Oculus retains events, logs, metrics
and traces; Agora carries attributable findings. The verifier classifies
product, test, harness, environment and inconclusive outcomes, never retries
silently into green, and cannot waive required evidence or human rollout
approval. A local model may select from the governed test catalog, interpret
failures and propose follow-up; the harness owns execution and evidence
admission.

## Product and dogfood requirements

- Use Vellum for working context and editable delivery artifacts, Chronos for
  the executable dependency graph and operator agenda, Moira for governed
  delegation and gates, Agora for collaboration, Dendrite for bounded ambient
  expertise, Daimon for execution, and Moira observations for events, logs,
  metrics, and traces.
- Exercise TUI and headless routes. Keys, mouse, command palette, headless
  commands, and future APIs converge on the same behavior facade and authority
  contracts.
- Providers and models, ACP sessions, work modes, keybinding profiles, context
  limits, budgets, and operational endpoints are discoverable interface
  configuration.
- Human and agent operators receive equivalent typed observations, command
  discovery, receipts, refusal reasons and safe-next-action guidance through
  TUI and headless surfaces. Agent ergonomics never bypasses human gates,
  least authority, bounded context or independent settlement evidence.
- The target dogfood topology is a durable Claude/Codex supervisory thread in
  an Agora room that watches Chronos and delegates fit-for-purpose bounded work
  through Daimon to local models. Persistent SRE and delivery-management agents
  may observe Oculus/Moira and Chronos and propose incidents, backlog changes,
  prioritization or follow-up; proposals remain owner-bound, attributable and
  human-gated wherever a credible stakeholder could contest the effect.
- Every action exposes pending, success, refusal, and outcome-unknown settlement
  with correlation, retained evidence, and an actionable next step without
  leaking task or model content into telemetry.
- Identify interface friction continuously. Remediate journey blockers in the
  active slice; capture lesser friction as prioritized, evidence-linked Chronos
  work.
- Select models from task-specific evidence. Smaller or local models receive
  bounded Vellum packets and Dendrite manifests, independent verification,
  escalation conditions, and human review. Rejected work is categorized as
  context, harness, decomposition, role, or model-fit failure.
- Stakeholder ensembles begin with: do we have the right ensemble? Preserve
  equitable independent first contributions, surface dissent, and require human
  gating wherever a credible human advocate could contest a consequential
  decision.
- No production rollout, destructive migration, credential or authority
  expansion, or consequential go/no-go proceeds without the named human gate.

## Operating reviews and sponsorship

Pantheon must make alignment review available through the UI, not only through
conversational narration.

- The execution journal records turn-level outcomes, evidence, friction,
  surprises, decisions, and next slices.
- Every per-turn reconciliation appends to that journal and becomes traceable
  source material for the next MBR or QBR. The UI shows the current review
  period, accountable sponsor, outstanding alignment questions, due human
  gates, and the evidence behind each claimed outcome.
- The monthly business review compares outcomes with commitments, journey and
  operational health, user-friction trends, model and provider performance,
  risks, accountable owners, and sponsor asks.
- The quarterly business review and project proposal records strategic
  alignment, investment case, roadmap and dependency changes, stakeholder-
  ensemble adequacy, alternatives and dissent, and an explicit human-gated
  go/no-go.
- Every MBR and QBR has a named accountable executive sponsor. Vellum owns the
  review artifact, Chronos schedules preparation and sign-off, and the operator
  agenda surfaces due approvals and unresolved decisions.
- Review completion requires an inspectable sign-off or refusal. Silence,
  timeout, relabeling, and agent impersonation cannot satisfy a human gate.

## Product alignment control loop

An intelligent product manager is a first-class stakeholder between the
external human advocate and any Pantheon-constrained frontier supervisor. The
role continuously reconciles the governing goal revision, acceptance evidence,
priority, dependency state, work leases, drift, grooming history and pending
sign-off. It may propose decomposition, sequencing, acceptance criteria and
priority changes; it may not silently rewrite the goal, accept its own work,
suppress stakeholder dissent, or assume executive, product-owner or rollout
authority.

The external frontier collaborator and a long-running Pantheon-constrained
frontier insider are separate attributed principals, even when they use the
same model family. They align through owner facts rather than private agent
memory: each executable slice has one current Chronos task/attempt/lease; each
Agora contribution names its participant; and each delivered effect carries
its owner receipt. The product manager detects overlapping claims, stale goal
revisions, conflicting recommendations and abandoned work, then proposes a
boundary, pause, handoff or escalation. It cannot dispatch an unadmitted task,
transfer a lease, merge two attempts, or declare one agent's narration to be
the other's evidence. A disagreement that changes scope, priority, acceptance
or consequential risk goes through attributed Agora challenge and the
applicable product-owner, sponsor or human gate.

The minimum recurring, owner-bracketed reconciliation cycle is:

1. Observe the exact Vellum goal revision and current Chronos graph, attempts,
   leases, dependencies and prior grooming receipt.
2. Ask Moira whether the represented stakeholder ensemble remains adequate;
   retain dissent rather than averaging it away.
3. Join only current owner evidence from Tellus, Oculus, budget-policy and Aegis
   boundaries; unavailable or incompatible owners make the frontier incomplete.
4. Compare commitments, active claims, outcomes, friction and drift across the
   external collaborator and constrained insider.
5. Publish a PM-authored Vellum proposal and open an attributed Agora challenge
   when material judgment is involved.
6. Apply only the separately authorized owner/human disposition to Chronos and
   retain the exact mutation receipt. Re-observe before the next dispatch.

The cycle runs on relevant owner events and at a bounded scheduled cadence;
event bursts are coalesced, and an unchanged frontier does not manufacture a
new proposal. MBR/QBR review provides the slower strategic control loop and
named executive sponsorship, not a substitute for operational reconciliation.

Chronos is the authoritative scheduling and alignment index. Rich objective,
acceptance and review content remains in Vellum under exact artifact references;
Agora carries attributed challenge; Tellus carries grounding/provenance; Oculus
carries observed behavior; Moira retains the selected ensemble and deliberation;
and Aegis carries effect authority and human gates. A reconciliation must name
the exact owner revision it observed and distinguish a proposal from an applied
owner mutation.

Pantheon's operator agenda imports current owner-published dependency-blocked
and uncertain Chronos work. Blocked work is inspect-only: it shows owner
priority and prerequisite count and routes the operator to the exact Chronos
dependency view. Uncertain work remains a separate recovery condition. Agenda
presence never changes priority, completes a prerequisite, grants a lease or
accepts an outcome. This attention projection is the first operational PM
surface; autonomous grooming and acceptance remain unavailable until the
owner-bound worker, task-linked conversation, model-role-fit and stakeholder-
ensemble contracts are qualified.

The agenda also imports Chronos's read-only product-alignment observation.
Owner-reported proposal drift and exact pending human-gate references retain
their workstream, alignment-record revision and owner resource revision and are
inspect-only. When a current direct Moira pending gate is composed, its richer
decision material supersedes the alignment reference; alignment drift remains
independently visible. An unavailable alignment source invalidates the
projection instead of licensing cached PM advice. Neither projection grants a
response control, creates a ticket, changes priority or applies a proposal.

Qualification must cover both the interactive agenda and a concurrently
attached headless client against the same owner. A passing isolated fixture does
not erase a live owner's attachment refusal; the product manager treats that
disagreement as drift requiring explicit follow-up, not as permission to use a
cached projection.

## Cyclic capability, boundary and learning review

Every meaningful turn reconciliation, MBR and QBR includes a productive
introspection/reflexion pass. It must improve the delivery system as well as the
current deliverable without turning Pantheon into a universal semantic owner.
The required fractal review geometry, Engram/Pies prior-art transfer protocol,
standards anchors, initial transfer map and placement refusals are maintained in
[`fractal-capability-review.md`](fractal-capability-review.md).

- Inventory existing peer and host capabilities before proposing new code.
  Record capabilities that are unused, duplicated, difficult to discover,
  incompletely composed, or missing TUI/headless qualification.
- Review legitimate surface gaps across owner events, diagnostic logs, metrics,
  traces, profiles, alerts, domain receipts, user feedback and UI
  instrumentation. Preserve source, schema, population, sampling, cardinality,
  retention, gaps, time quality and semantic limits; telemetry may orient work
  but cannot prove authority, a durable effect or journey completion.
- Ground recommendations in current owner contracts, live dogfood evidence,
  applicable standards and established domain practice. State where precedent
  does not fit Pantheon's bounded ownership model instead of copying it blindly.
- Apply reuse and leakage tests: one semantic fact has one sovereign owner;
  projections retain owner identity and revision; common presentation does not
  imply common authority; and host convenience cannot become a shadow protocol.
- Consider promotion into Keystone only for a small mechanism or law that has
  repeated independent consumers, identical semantics, bounded failure modes
  and cross-consumer qualification. Domain events, decisions, routing, retry,
  reconciliation, UI behavior and Pantheon composition policy remain outside
  Keystone.
- Treat Hermes as ordered domain-event delivery, not a generic log, metric,
  trace or audit store. Correlation may link Hermes cursors to other signals,
  but must not reinterpret their evidence or disclosure policy.
- Flow reusable lessons deliberately: Vellum retains the review artifact;
  Dendrite grounds standards, precedents and reusable expertise; Moira retains
  bounded deliberation, dissent and finding disposition; Chronos owns accepted
  follow-up work; Hermes carries owner-published domain events; Moira
  observations and the selected telemetry owners expose content-safe operating
  evidence.
- For each material finding, record reuse/remediation/defer/reject disposition,
  rationale, accountable owner, evidence, next review trigger and whether a
  named human or executive decision is required.
- Apply the review across breadth, depth, up, down, in, out and lifecycle time.
  Bound recursion by materiality, novelty, risk and evidence. Prior art is
  classified as direct reuse, owner-preserving adaptation, presentation
  inspiration or rejected warning before any transfer into Pantheon.
- Run the review through Rune's existing audit, gap-analysis, prior-art,
  ensemble, instrumented-BDD and user-acceptance machines where applicable.
  Use `scry`, `machine_path_check`, `smith` and Dendrite for the structural and
  corpus claims they actually prove; disclose semantic, runtime and human-gate
  gaps rather than promoting a structurally valid machine to delivery evidence.
- Find and state the smallest load-bearing truth at the center of each material
  review. Drive `maybe` toward scoped Boolean proof/refutation when the claim is
  decidable, or toward explicit mu/named-human judgment when meaning, values or
  legitimate plurality cannot be reduced honestly. Consensus, confidence and
  telemetry never substitute for either.

## Overnight execution boundary

Work proceeds in bounded, reviewable commits. Capability promotion requires
scope-matched live evidence, headless and TUI parity where claimed, restart or
durable-reopen proof, and synchronized peer repositories. Overnight execution
may continue through ordinary implementation and verification, but stops at
destructive actions, production rollout, ambiguous authority expansion, or
decisions reserved for a human sponsor.
