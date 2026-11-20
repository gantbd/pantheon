# Rune blueprint-publication composition review

Date: 2026-11-20

Status: proposed composition awaiting named-human ensemble-fit and topology
decision; no admitted Rune topology, generated projection, Vellum mutation,
task effect, go/no-go or rollout authority

## Exact owner question boundary

The lossless source is
`design/requests/rune-blueprint-publication.blueprint.json`. Its task prose asks
for the smallest in-memory bridge from Pantheon's existing question-bound
blueprint adapter into its existing human-gated minimal Vellum publisher. It
explicitly refuses duplicated authoring, publication, approval or Vellum
semantics and retains the stronger high-risk and Gray admission contracts.

The first real `pantheon rune-blueprint` run used the pinned current Rune
author, Scry, Smith and rustc toolchain. Starting from `request: null`, Rune
returned exactly three questions and no candidate, typed IR or projection:

1. `$.holes` — which semantic holes remain unresolved;
2. `$.review` — who reviewed the exact request and what the ensemble-fit answer
   is; and
3. `$.machine` — what machine topology should be authored.

The normalized empty request digest was
`sha256:557eba451605c31c1ee56aea2f77e3e3e5cff17605d6eed95c66b3505e319cf9`.
The run classified the prose as `retained_not_inferred`, returned
`questions_pending`, and marked publication
`not_eligible_until_projection_complete`. This review can prepare the answer;
it cannot answer the named-human question or mutate the request under the same
digest.

## Reusable owner boundaries

- Rune owns question meaning, legal machine semantics, Scry validation and the
  Smith projection. Pantheon must not synthesize an owner answer or weaken an
  unsupported construct.
- `rune-blueprint` owns exact task retention, question-bound request revision,
  additive risk gating and the distinction between prose LST and request AST.
- `rune-packet-publication` owns deterministic projection-packet construction,
  Vellum target observation, minimal UTF-8-safe patch derivation, exact human
  confirmation, single-dispatch uncertainty and durable-save recovery.
- Vellum owns document membership, revision/edit truth and persistence.
- Pantheon may compose exact outputs in memory. It gains no semantic, human,
  Vellum, task, go/no-go or rollout authority by doing so.

## Options

### 1. Thin in-memory composition over both existing adapters — recommended

Refactor each adapter to expose one private typed preparation result while
keeping its current CLI behavior. A new composition command invokes blueprint
preparation, requires the exact owner-complete and risk-eligible result, and
passes the exact revised request bytes and already-qualified owner outcome into
packet preparation. Preview and publish independently recompute the whole
chain. No temporary semantic file is created.

Pros: removes the current manual request-file handoff; reuses both qualified
boundaries; keeps one risk decision and one publication implementation; binds
the human challenge to the exact recomputed packet and Vellum target.

Cons: requires careful private API extraction and a new composition FSM; stage
observations must be joined without treating telemetry as authority; the CLI
needs unambiguous answer and approval arguments.

### 2. Private temporary request-file handoff

Serialize the revised request into a mode-0600 temporary file and invoke the
existing publisher unchanged.

Pros: minimal Rust refactoring and maximal reuse of current public command
shape.

Cons: creates avoidable materialization, cleanup and provenance obligations;
turns an in-memory value into filesystem state; retains deterministic toil in
the operational path. Useful only as a diagnostic bridge, not the stable core.

### 3. Teach the packet publisher to parse blueprints and answers

Move question revision and risk checks into the publisher.

Pros: one command and superficially direct control flow.

Cons: duplicates or absorbs `rune-blueprint`, couples mutation to authoring,
and invites policy divergence. Rejected.

### 4. Introduce a generic pipeline/workflow engine now

Model every stage as a configurable generic executor.

Pros: may eventually generalize to other Rune-authored workflows.

Cons: adds a new abstraction and configuration authority before this exact
composition is proved; increases semantic and security surface; risks replacing
owner contracts with stringly orchestration. Deferred until repeated evidence
demands it.

## Proposed version-one topology for review

The candidate topology is additive around the two existing machines:

1. `blueprint-received`
2. `questions-returned`
3. `projection-qualified`
4. `risk-eligible`
5. `publication-previewed`
6. `publication-armed`
7. `patch-accepted`
8. `persistence-pending`
9. `durable-published`
10. `outcome-unknown`
11. `refused`

Question-bound revisions loop only from `questions-returned` back through fresh
blueprint preparation. `projection-qualified` cannot advance unless Rune is
complete and Pantheon's additive policy reports the literal low/moderate
publication eligibility. High and Gray terminate in `refused` with their named
missing contract; they do not enter preview. `publication-previewed` and later
states reuse the existing publisher transitions rather than restating their
semantics. Preview cannot mutate. Publish recomputes from the original
blueprint, exact answers and current deployment profile; stale request, risk,
packet, target, patch or approval evidence refuses before mutation.

The only successful effect path is
`blueprint-received → projection-qualified → risk-eligible →
publication-previewed → publication-armed → patch-accepted →
durable-published`. Questions, refusal, unknown edit and pending persistence
remain distinct. No implicit retry or rollback edge exists.

## Threat and leakage review

- **Semantic substitution:** bind answers to the exact request digest and exact
  Rune question tuple; pass the revised bytes directly rather than reconstruct
  them from presentation JSON.
- **Risk bypass:** require the typed internal eligibility value, not a caller
  string or inferred risk class. Recompute it during publish.
- **TOCTOU:** recompute blueprint, owner outcome, packet, Vellum snapshot and
  patch before validating approval; retain the existing revision fence.
- **Authority laundering:** advisory SME output cannot populate human review;
  a qualified projection cannot authorize publication without the separate
  exact human challenge.
- **Duplicate semantics:** the composition controller may sequence existing
  results but cannot redefine Rune questions, packet identity, patch selection,
  Vellum truth or durable recovery.
- **Sensitive material:** blueprint prose, answers, projection bytes, target
  content, confirmation and credentials stay out of logs and metrics.
- **Uncertain delivery:** preserve the existing no-retry edit and durable-save
  recovery contracts unchanged.

## Instrumentation and value proof

The composition should emit one content-free observation with correlation and
stage durations for blueprint parsing, Rune authoring, deterministic answer
application, risk admission, packet preparation, Vellum observation, approval
validation, edit and persistence. It should retain counts for owner questions,
answers, deterministic revisions, refusals, packet/target/inserted bytes and
configured saturation. Token/context cost and escaped defects remain
`not_observed` until a real owner provides them. `oculus_published=false` stays
explicit until Oculus receives the record.

The real-slice comparison must measure the current manual request-file handoff
against the composed path: operator steps, elapsed lead time, pre-code findings,
rework operations, context/token cost where observable, refusals and escaped
defects. A passing test without that comparison is qualification evidence, not
proof of value unlock.

## Advisory evidence and proposed stakeholder ensemble

An isolated authenticated Dendrite owner loaded `rune/sme/triggers` and
Pantheon's concern lattice. Existing qualified architecture and verification
plans selected rigor, architecture, network, identity, data, security,
user-experience, humanism, distributed-systems, Rust engineering, test-harness
engineering and live verification. Their correlated concerns remain visible.
These roles are advisory lenses, not stakeholders or approval principals.

The proposed equitable stakeholder ensemble is:

- Brian, named human sponsor/operator and only version-one design gate;
- Rune owner, accountable for questions, machine legality and projection;
- Vellum owner, accountable for document/edit/persistence truth;
- Pantheon owner, accountable for exact composition and presentation only;
- affected human and agent user advocacy, represented by Brian for this bounded
  single-user slice and requiring reassessment as the audience grows;
- independent security/adversarial and live-verification challengers, with
  dissent standing but no human-approval authority; and
- Oculus/observability ownership as a consulted stakeholder for the telemetry
  contract, without making telemetry availability an admission condition.

Equity means every owner may refuse misuse of its boundary, challengers retain
attributed dissent, and model or owner consensus cannot outvote the named human
gate.

## Human gate

The named sponsor must answer both questions before the topology is encoded in
a Rune author request:

1. Do we have the right ensemble for this bounded composition slice?
2. Do you approve option 1, the thin in-memory composition over the existing
   blueprint and packet-publication adapters, as the version-one topology to
   author and qualify?

Approval answers only these design questions. It is not acceptance of any
generated design, authorization to publish a particular packet, Chronos
completion, go/no-go or rollout approval.
