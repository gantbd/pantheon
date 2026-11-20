# Rune blueprint-publication composition review

Date: 2026-11-20

Status: named human sponsor approved the ensemble and option-one topology;
source-level invoked-child preservation qualified at Rune commit `23bac56`;
the thin composition Rune validates and has a complete Smith Rust projection;
the in-memory runtime adapter and one owner-backed manual-versus-composed slice
are qualified; no task effect, design acceptance, go/no-go or rollout authority
follows

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

## Encoding decision and compiler prerequisite

The named sponsor approved Rosetta/Rune as the semantic encoding rather than
the large JSON authoring document exposed during the first topology pass. The
durable law is now:

`reviewed Rosetta → canonical .machine.rune → Scry typed IR → Smith Rust`

JSON may remain a closed generated transport envelope where the current author
or adapter API requires it. It is not the durable topology, does not become a
second semantic source, and should be discarded after the canonical Rune and
lineage identities exist.

This decision exposed a legitimate critical-path compiler gap. Rune's typed IR
already represents `(invoke F)`, Smith's preservation law explicitly requires
unresolved or refused child settlement to absorb the parent transition, and
the compatibility baseline retains Gleam's existing invoke behavior. However,
Scry's current `.rune` edge grammar does not admit an `invoke` clause, and the
Rust target correctly reports invoked-machine settlement as unsupported. The
desired holofractal source therefore cannot yet be projected into the selected
Rust target without one of three weaker substitutions:

- flatten the packet-publication states into the parent, duplicating their
  semantics;
- use `(ref depends-on …)`, which proves dependency/provenance but not runtime
  child settlement; or
- perform an opaque Rust adapter handoff outside the compiled obligation set.

All three are rejected as completion claims. The recommended prerequisite was
to close the narrow source-to-Rust invocation seam: admit and validate an exact
edge child invocation in Scry, lower it losslessly into the existing typed IR,
make Smith Rust require an explicit child success/refusal/unknown verdict before
parent advancement, and prove that nesting or reassociation cannot launder a
non-success child into parent success. Existing target behavior must not be
reduced, and unsupported targets must continue to refuse rather than emit
weaker executables.

That prerequisite qualified on 2026-11-20 at Rune commit `23bac56`. Scry now
admits one singular `(invoke CHILD)` edge clause and lowers it into the existing
typed IR. Smith Rust requires the runtime adapter to return `Succeeded`,
`Refused`, or `Unknown`; refused and unknown children remain typed parent
refusals and only exact success makes the edge eligible. The public
Rune→Scry→Smith→rustc→execution gate proves the behavior and rejects two
child identities on one edge. Recursive multi-level orchestration remains
unqualified and is not needed by this version-one parent.

Once that prerequisite qualifies, the composition Rune should remain thin:
its upstream states own blueprint questions, qualified projection and additive
risk eligibility; its publication edge invokes the existing
`pantheon-rune-packet-publication` machine. The child continues to own preview,
human arming, edit uncertainty and persistence. The flattened state list above
remains an end-to-end presentation of the composite trace, not a second source
of the child's laws.

The admitted source is
`design/runes/pantheon-rune-blueprint-publication.machine.rune`, with generated
Rust at `design/generated/pantheon_rune_blueprint_publication.rs`. Scry accepts
the exact source and Smith reports Rust compatibility `complete` with no
unsupported semantics. The parent contains only blueprint clarification,
projection qualification, additive risk admission, upstream refusal and one
invoked-child success edge. It does not copy the child's preview, arming, edit,
uncertainty or persistence states. Projection qualification is compiler
evidence rather than runtime proof. Pantheon's qualified adapter now supplies
that separate runtime evidence: it retains the exact canonical revised request
bytes in memory, verifies that the qualified owner result binds those bytes and
that the publication profile did not change between stages, and passes both to
the existing packet publisher without a semantic handoff file or a second Rune
owner invocation.

The generated parent remains authoritative for advancement. Unresolved
questions enter `questions_returned` without contacting Vellum. A complete
high-risk request enters `refused` without contacting Vellum. The owner-backed
gate now separately proves a complete Gray request retains
`caller_declared_not_verified`, names the missing sandbox, reversibility and
expiry experiment contract, and refuses with zero publication invocations.
Gray therefore remains executable Maybe evidence rather than borrowing the
moderate-risk rail. Preview leaves
the parent at `publication_eligible`, because a preview is not durable child
success. Publish advances to `durable_published` only when the invoked child
returns its exact durable success; refusal or unknown cannot be laundered into
parent advancement. The existing child still owns its human challenge,
revision fence, minimal UTF-8-safe patch, single edit dispatch and persistence
recovery.

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

The composition emits one content-free observation with total, blueprint and
downstream stage durations. It retains counts for owner questions,
deterministic revisions, refusals, publication invocations, semantic handoff
files and packet/inserted bytes. It does not yet claim per-substage timing for
parsing, authoring, risk, Vellum observation, approval, edit or persistence.
The record independently binds its Pantheon capability, generated machine
source/IR and exact request digest; declares one-invocation scope,
complete-per-invocation sampling and content-free disclosure; and lists every
unavailable measure and owner receipt as an explicit gap. Token/context cost
and escaped defects remain `not_observed` until a real owner provides them.
`oculus_published=false` stays explicit until Oculus receives the record.

The owner-backed real-slice gate compares the current manual request-file
handoff against the composed path using the same canonical revised request and
proves that both produce the same packet identity. In the observed run, the
manual path required two Rune owner invocations, one materialized semantic
handoff file and 4,875 ms; the composed path required one owner invocation,
zero semantic handoff files and 2,446 ms. That is one measured sample of about
50% lower preview lead time, not a universal benchmark. It proves removal of
one deterministic owner round trip and one avoidable materialization without
weakening packet identity. Token/context cost, pre-code findings, escaped
defects and longitudinal rework remain unobserved, and the record is not yet
published through Oculus.

The follow-up instrumentation gate now requires that shape on every composed
outcome rather than only success. Questions and previews report zero refusals;
typed risk and downstream-owner refusals report one. Every outcome retains
stage duration, owner-request, question, deterministic-revision, publication,
handoff and patch counts. Pre-code findings, rework operations, token/context
usage and escaped defects remain the literal `not_observed`; the adapter does
not restyle questions as defects or deterministic revision as rework. A second
owner-backed run measured 4,718 ms for the manual path and 2,397 ms for the
composed path while preserving the same two-to-one owner-request and one-to-zero
semantic-file reductions. Two samples corroborate direction, not a performance
distribution or SLO.

Pantheon then dogfooded the current writable Chronos TUI against the
authoritative roadmap. Revision 84 already contained the five-node value-core
chain, `oculus-observability-query-peer`, and
`chronos-external-work-admission-contract`; no duplicate tasks were admitted.
The delivered value-core nodes remain `open` or `blocked` because this session
has no legitimate external-delivery evidence/review transition. They were not
falsely claimed or marked done. The independently observed shared incremental
target stall was absent from both bounded pages, so the reviewed composer
admitted only `pantheon-shared-target-build-stall-diagnostic` at priority 5
with capability `build-diagnostics`, advancing Chronos to revision 85. The
separate `chronos-task-observe` gateway confirmed incarnation 123, revision 85,
status `open`, execution `ready`, one required capability and no blocker.

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

## Human gate decision

Brian, the named human sponsor, answered both questions on 2026-11-20:

1. Do we have the right ensemble for this bounded composition slice?
2. Do you approve option 1, the thin in-memory composition over the existing
   blueprint and packet-publication adapters, as the version-one topology to
   author and qualify?

The ensemble is fit for this bounded slice, and option 1 is approved as the
version-one topology. Approval answers only these design questions. It is not acceptance of any
generated design, authorization to publish a particular packet, Chronos
completion, go/no-go or rollout approval.
