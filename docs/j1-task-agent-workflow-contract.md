# J1 task-agent workflow instantiation contract

Status: proposed implementation contract

Governing journey: [J1 in the user-journey blueprint](use-cases-and-user-journeys.md#j1-create-a-ticket-that-invokes-an-agent-with-a-user-interaction-room)

Canonical Chronos work: `j1-dynamic-task-participation-routing`

## Decision

Moira owns the durable task-agent workflow instance and the owner-derived
identities that compose it. Pantheon may preview, confirm, observe and route to
owner commands, but must not construct a participation binding from UI state or
profile files.

The ownership split remains:

| Truth | Sovereign owner |
|---|---|
| Task graph, attempt and lease | Chronos |
| Workflow frontier, effect identities and settlement | Moira |
| Room, participants and messages | Agora |
| Agent session, turns, budgets and provider evidence | Daimon |
| Grounding identities and freshness | Dendrite/Tellus |
| Permission and effect authority | Aegis |
| Documents, proposals and accepted edits | Vellum |
| Relationship records whose semantics span owners | Selected relationship adapter/owner |
| Presentation, human confirmation and navigation | Pantheon |

This is not a new Pantheon orchestration engine. The Boolean center is one
durable owner-issued journey reference whose typed links resolve to independent
owner facts without copying their semantic state.

## Why the existing command is insufficient

Moira's current `agent_participation::MutationV1::Admit` is intentionally a raw
owner mutation. Its `BindingV1` requires a participation, workflow,
deliberation/member, session, profile, exact task attempt, method, grounding and
persona. Pantheon cannot safely derive those values merely because it can see a
Chronos task and configured provider.

The current fixed-participation profile and one-step owner worker are valid for
qualification and explicitly configured operation. They do not provide a
normal-user task-to-workflow instantiation contract. Treating their opaque
configuration as a template registry would duplicate owner truth and make
headless behavior diverge from the TUI.

## Required owner records

### `TaskAgentTemplateV1`

An immutable, versioned Moira-owned definition containing:

- opaque template identity, version and canonical digest;
- supported work mode and participation method;
- room admission and reviewer policy;
- permitted agent-profile selection constraints;
- context, tool, token, elapsed-time and effect ceilings;
- required grounding depth and freshness policy;
- required Aegis policy references and human gates;
- compatible schema/capability ranges;
- deprecation and retirement state.

It contains no credential, bearer grant, executable path, endpoint, provider
secret or mutable peer state.

### `TaskAgentJourneyV1`

A durable Moira-owned instance containing:

- one owner-issued stable journey identity;
- exact template identity, version and digest;
- initiating principal and accountable human owner;
- typed references to the Chronos task/attempt, Agora room, Moira workflow and
  participation, Daimon session/profile, grounding manifest, Aegis decisions,
  Vellum artifacts and relationship records as they become available;
- current workflow frontier and every retained prior frontier;
- one operation identity and disposition per cross-owner effect;
- creation/observation times, owner revisions and explicit missing or stale
  links;
- human review/confirmation evidence without embedding authority credentials.

The record references peer facts. It does not restyle a room message as task
progress, a provider result as completion, or a human confirmation as authority.

## Commands

Names are descriptive until Moira publishes the transport-neutral protocol.

### `PreviewTaskAgentJourneyV1`

Input:

- exact template reference and expected version/digest;
- exact Chronos task admission reference;
- requested agent profile, room policy and optional reviewer references;
- operator principal and accountable human owner;
- bounded user-authored objective/acceptance references, not copied documents;
- current owner evidence references needed to validate the selection.

Output is a content-bounded preview with:

- selected owners and capabilities;
- expected effects and human gates in order;
- budget and grounding bounds;
- conflicts, missing evidence and unsupported selections;
- a canonical preview digest and expiry;
- one next safe action.

Preview creates no workflow, room, session, lease, grant or participation.

### `InstantiateTaskAgentJourneyV1`

Input:

- the exact unexpired preview digest;
- expected template and relevant owner revisions;
- a caller-supplied idempotency identity;
- an exact human confirmation binding the preview and consequence.

Moira validates current evidence and derives the workflow, effect and
participation identities. Matching replay returns the original journey;
changed input under the same identity conflicts. Any stale witness refuses
without retargeting.

The accepted result proves only durable workflow admission. It does not claim
that an agent, room or provider call already exists.

### `ObserveTaskAgentJourneyV1`

Returns the current owner frontier, typed relationship references, per-owner
freshness/revision, pending human/authority actions, uncertainty fences and one
owner-supported next safe action. TUI and headless clients consume this same
projection.

### Owner-derived progression

Subsequent commands name only the journey and expected frontier/revision. Moira
derives exact effect identities and submits them through selected trusted
receivers/workers. Pantheon never submits a raw participation binding, Chronos
lease, Daimon session envelope or Agora room mutation on the workflow's behalf.

## Minimum state model

1. `Previewed` — read-only, expiring and non-durable unless explicitly retained.
2. `Admitted` — journey identity and template binding are durable; no peer
   effect is implied.
3. `TaskAttemptReady` — Chronos has selected an exact current attempt under its
   trusted worker/lease law.
4. `RoomReady` — Agora has created or resolved the exact room and the selected
   relationship owner has retained the task-room link.
5. `ParticipationAdmitted` — Moira has derived and durably admitted the binding
   against the exact attempt, method, grounding and profile.
6. `OpenPrepared` — the owner-derived Daimon open effect is durable.
7. `AgentReady` — the exact Daimon session is owner-observed Ready.
8. `Active` — addressed messages may be admitted through explicit replay-safe
   prompt commands; ordinary chatter is inert.
9. `ProposalReview` — typed proposals and evidence await separate owner/human
   disposition.
10. `Settled`, `Cancelled`, `Failed` or `FencedUnknown` — terminal semantics are
    explicit; uncertainty is never rewritten as failure or success.

The implementation may refine these phases, but it may not collapse owner
effects or skip the distinction between admission, preparation, dispatch and
observation.

## Refusal and recovery laws

- Missing, ambiguous, stale, unauthorized or conflicting owner evidence refuses
  before the next effect.
- A lost reply after possible dispatch enters `FencedUnknown`; blind retry is
  unavailable until owner lookup/replay reconciles the exact operation.
- Matching replay cannot create another task, room, participation, session,
  provider call, edit or message.
- Provider health, task completion, room activity, agent output, proposal
  acceptance and production approval remain distinct facts.
- Cancellation, session close, workflow cancellation, compensation and room
  archive are separate controls with separate receipts.
- Profile/template changes do not retarget an admitted journey. A material
  change creates an explicit migration or replacement decision.
- Human confirmation is not an Aegis grant. Authority is checked again at the
  command owner immediately before each protected effect.

## Pantheon presentation contract

The TUI and headless clients use the same behavior facade and owner commands.
The interactive path provides:

- task creation followed by an explicit “delegate through workflow” action;
- template/provider/room/grounding/budget selection through owner catalogues;
- review → arm → confirm of the exact preview;
- a stable journey view linking task, workflow, room, participation, session,
  grounding, authority, proposals and receipts;
- observable success/refusal/uncertainty with an actionable next step;
- navigation back to the Chronos board without losing the journey reference;
- a unified attention entry for permission, review, uncertainty and human gate
  states;
- no hidden raw shell, profile edit or environment-variable prerequisite in the
  normal user path.

Headless execution requires an explicit noninteractive challenge that binds the
same preview, revisions, scope and consequence. It never defaults an ambiguous
template, provider, room, human owner or authority choice.

## Implementation cuts

Current bounded safeguard (2026-11-19): Pantheon re-observes the exact active
Chronos workstream immediately before Daimon invocation and requires the
owner-projected task to be Running at the exact configured incarnation and
attempt. Ready, stale, missing and mismatched tasks refuse before Daimon with a
Moira/trusted-worker next step. This is a presentation-boundary safety check,
not workflow instantiation: it creates no attempt, lease, effect, participation
or session identity and does not settle Chronos.

Moira `02e59ac` implements the first portion of cut 1 in its sovereign core:
bounded immutable templates, exact expiring previews, trusted-time named-human
confirmation, owner-scoped journey/workflow allocation, one-journey-per-preview
and canonical checkpoint/reopen. It deliberately exposes only `Admitted`; the
transactional store/service/transport and every peer-effect transition remain
required before Pantheon may compose this path.

Moira `dacd31c` added the deterministic exact-claim law; `9ddb85f` now closes
its durable owner transition without exposing a client mutation. The trusted
Chronos worker claims once and immediately submits the exact receipt to the
journey store. Moira transactionally advances `Admitted` to
`TaskAttemptReady`, retains task key, zero-based incarnation, attempt and newer
Chronos revision, and projects that evidence through its read-only protocol.
Matching replay does not advance storage; altered evidence conflicts. If
persistence conflicts or becomes indeterminate after the Chronos claim, the
error retains the successful claim and a bind-only recovery path reuses it
without a second Chronos mutation. Canonical checkpoint V2 reopens the phase
and upgrades admitted V1 checkpoints. A real Chronos server plus SQLite test
proves normal progression, restart, conflict recovery and zero additional
worker requests during rebind. This remains claim evidence, not a lease bearer,
Aegis delegation, participation, session, provider invocation or completion.

Moira `b46ef62` removes caller-supplied observation/expiry/confirmation time:
clients request a bounded TTL and the owner retains trusted time. Moira
`28fae97` publishes the independent transport-neutral DTO and canonical wire
families for catalogue, preview, instantiate, preview/journey observation and
status. Moira `72b87bb` adds the transactional SQLite store and service-owned
trusted-time admission path; canonical checkpoints survive reopen, matching
preview/instantiate operations replay without advancing storage, stale writers
conflict, and ambiguous/lost ownership fences the store. Moira `c7fe06d` exposes
that same contract through bounded local daemon and capability-shaped client
facades, with a real Unix test proving catalogue, preview, named-human-bound
instantiate, restart, observation and exact replay.

Pantheon's headless composition now consumes only those owner facades. Its
bounded commands catalogue templates, create a preview from a size-limited
versioned document, emit an exact human confirmation challenge, instantiate
only when that challenge matches, and observe the durable journey. Every JSON
result carries owner/storage revisions, replay or uncertainty disposition,
boundary language and an actionable next step. A process-level qualification
runs the real Pantheon binary against a restarted Moira SQLite owner for every
step. Its observer now renders `TaskAttemptReady` and the exact retained claim
when Moira publishes it, while keeping Aegis delegation, participation, Daimon
session and completion explicitly absent. This completes cut 1, the headless
admission portion of cut 3 and the first trusted evidence transition. Current
Aegis delegation, later owner-derived effects, aggregate profile configuration,
operator-agenda projection and TUI parity remain open.

### Reuse boundary

The template, budget, grounding, authority, accountable-human, replay and
persistence laws are reusable for any task-backed SME role. Continuous SRE
observation is a different ingress profile: Oculus owns telemetry query/cursor/
gap facts, Hermes may carry minimized wake events, and deterministic policy
coalesces recurrence before a model is involved. It may hand a warranted task
to this journey after Chronos admission; it must not create synthetic tickets
for every signal or treat model interpretation as incident, paging, priority or
closure authority.

1. **Moira owner contract:** publish the template catalogue, preview,
   instantiate, observe and exact-replay protocol plus durable store. Reuse the
   existing participation, proposal-saga and worker contracts; do not fork them.
2. **Trusted evidence adapters:** validate Chronos task/attempt facts, Daimon
   profile/session facts and task-conversation links without serializing worker
   authority or credentials into the journey.
3. **Pantheon composition:** add the discoverable guarded action, minibuffer
   flow, journey pane, headless preview/confirm commands and operator-agenda
   projection using only the owner protocol.
4. **J1 live gate:** create a task, instantiate the workflow, create/bind the
   room, open the agent session, admit an addressed turn, retain a typed
   proposal, return to Chronos, and observe evidence-backed status/comment
   disposition across process restart.
5. **Recovery matrix:** exercise stale preview, denied authority, pre-dispatch
   refusal, lost accepted reply at every peer boundary, exact replay, changed
   replay conflict, cancellation/revocation races, capacity refusal and clean
   TUI/headless detach/reattach.

## Acceptance evidence

The contract is not complete until evidence proves:

- no Pantheon-generated participation/workflow/session/effect identity crosses
  the owner boundary except the caller's scoped idempotency identity;
- one exact task produces one durable journey and no duplicate peer effects
  under replay or restart;
- every displayed status names its owner, revision/freshness and uncertainty;
- TUI and headless clients produce the same owner requests and dispositions;
- an addressed Agora message can advance the exact session while ordinary room
  chatter cannot;
- an agent proposal cannot update Chronos, Agora or Vellum until separately
  accepted through the appropriate owner and authority gates;
- the operator can return to Chronos and resolve the same journey, status and
  attributable comments;
- final rollout and any material human-advocate finding still require a fresh
  named-human decision.

