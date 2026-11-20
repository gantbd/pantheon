# Rune packet publication ensemble review

Date: 2026-11-20

Status: human ensemble-fit decision required; no publication machine or
mutation capability is admitted yet.

## Stable core under review

The proposed first publication slice writes one complete deterministic
Rune-author projection packet into one pre-admitted Vellum document. It does
not yet extract the packet into separate source and Rust artifacts. This keeps
the first mutation boundary to one revision, one whole-document replacement,
one human confirmation and one durable operation identity.

The two phases are intentionally separate:

1. `preview` reruns the bounded Rune author, requires
   `projection_qualified`, constructs the deterministic packet, observes one
   configured Vellum document and returns the packet plus an exact confirmation
   challenge. It cannot mutate.
2. `publish` reruns every qualification and observation, requires a private
   exact human confirmation, sends one revision-bound Vellum workspace edit,
   and only after an accepted edit sends one durable save under the already
   bound operation identity.

An unknown edit outcome fences persistence and is never retried. An accepted
edit followed by refused or unknown persistence is `persistence-pending`, not
success and not a claim that the edit did not happen. Vellum's existing durable
receipt lookup remains the recovery owner.

## Rune author questions and proposed answers

The exact request at
`design/requests/rune-packet-publication.rune-author.json` has SHA-256
`b9e285227256489ba6ebacb0fddbaf2029e51662f073852318d7f58f947807b1`.
The first real Pantheon-to-Rune run returned eleven questions and no candidate.
After the owner answers and complete topology below were authored with
`ensemble_fit=contested`, the same owner boundary reduced its result to exactly
one question—`$.review.ensemble_fit: What must change before this ensemble is
fit for this exact design?`—and still emitted no candidate, typed IR or
projection bytes.

- Packet identity: deterministic pretty JSON containing schema/version, exact
  author request digest, Rune-author/Scry/Smith/rustc identities, confinement
  declaration and the closed `projection_qualified` owner outcome. Runtime
  duration and transport observations are excluded from packet identity.
- Vellum target: the human/deployment-owned profile names one document ordinal,
  socket and private credential. The adapter refreshes inventory, proves that
  exact ordinal is already a member, selects it and observes its complete
  current revision, byte length and SHA-256. No locator or membership operation
  is accepted from the command caller.
- Human approval: a private closed approval document binds the packet digest,
  workspace/document identity, current revision and digest, replacement digest,
  durable operation identity and exact generated confirmation. Version one
  proves exact operator confirmation, not cryptographic human identity; a
  Moira/Aegis-backed human principal is a later strengthening.
- Edit uncertainty: `OutcomeUnknown` returns `edit-outcome-unknown`, sends no
  durable save and permits no blind retry. Resolution requires authoritative
  Vellum re-observation or a future owner receipt contract.
- Persistence outcomes: only Vellum's exact durable outcome is `durable`.
  Refused, retired, request-conflict, absent/foreign, unavailable,
  completed-not-durable and outcome-unknown all retain the accepted dirty edit
  as `persistence-pending` with the exact operation identity and lookup action.
- Observability: content-free classification, duration, request/error count,
  phase, packet/target byte counts, configured limits, workspace/document
  ordinals, revisions, operation identity and digests. Packet bytes,
  confirmation, credential, locator and document content never enter logs or
  metrics. `oculus_published=false` remains explicit until that owner exists.

## Proposed machine topology

States:

1. `publication-request-received`
2. `preview-returned`
3. `publication-armed`
4. `in-memory-published`
5. `edit-outcome-unknown`
6. `persistence-pending`
7. `durable-published`
8. `refused`

Signals:

- `preview-returned`
- `publication-armed`
- `edit-accepted`
- `edit-outcome-unknown`
- `persistence-pending`
- `durable-published`
- `publication-refused`

The only mutating path is
`publication-request-received → preview-returned → publication-armed →
in-memory-published → durable-published`. Every edge binds fresh exact evidence.
The uncertainty and pending states are terminal in version one; recovery is a
separate owner operation rather than an implicit retry.

## Advisory ensemble evidence

Pantheon's real authenticated Tellus client queried an isolated Dendrite owner
using `design/requests/rune-packet-publication.sme-plan.json`.

- Fold: `rune/sme/triggers`, SHA-256
  `88e4aa16f24cc451010c2c1b8a99383bc19e3ff4f155e2a482a72871c5a239fb`
- Pantheon lattice SHA-256:
  `7e065399a0df6d36e6f43bf6a660b40b06715bfc40ada5ba7e27586bd8907fbf`
- Roles: identity, security, data, observability, humanism and live verification
- Mechanical result: sufficient roles, all pairwise orthogonal, status
  `current`

The first request correctly refused because Rune observability lacked a concern
pair. Rune commit `49ea55d` now registers the role-owned
`system-observability / causal-attribution` pair; Rune corpus self-assessment
and all 60 Smith grounding roundtrips pass after that repair.

## Stakeholder ensemble and dissent

SMEs are advisory lenses, not stakeholders. The proposed equitable stakeholder
ensemble is:

- Brian, accountable human sponsor and only version-one approval principal;
- Rune owner, accountable for packet semantics and refusal meaning;
- Vellum owner, accountable for workspace identity, edit truth, persistence and
  recovery;
- Pantheon owner, accountable only for exact composition and presentation;
- affected human/operator advocate, represented directly by the sponsor for
  this bounded single-user slice, with Humanism advice retained separately;
- independent security and live-verification challengers, without approval
  authority.

Dissent retained for the decision:

- A literal confirmation does not authenticate a human principal. It is an
  acceptable local operator gate only if its limitation is visible and the
  capability is not exposed to an agent.
- One packet document is provenance-first and deliberately less convenient
  than immediate multi-file materialization.
- Vellum has owner-atomic in-memory multi-document editing but not one atomic
  durable transaction across documents; claiming otherwise would weaken the
  design.
- A post-edit persistence refusal cannot roll back or erase the accepted dirty
  edit and must remain visible as pending.
- Neither successful publication nor durable persistence is acceptance of the
  generated design, authorization to execute it, go/no-go or rollout approval.

## Human gate

The exact question is: **do we have the right ensemble and the right stable
core for this bounded version-one publication machine?**

An affirmative decision admits topology authoring and another exact Rune-author
run. It does not approve implementation, publication of any particular packet,
agent exposure, or rollout.
