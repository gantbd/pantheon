# Chronos human review-disposition design verification

Date: 2026-11-20

Status: canonical owner-contract design admitted; owner implementation and live
effect qualification remain pending.

## Stable center

The review operation is not a generic status transition. It preserves four
different facts that the current engine partially conflates:

- an agent attempt reached execution success;
- a named human reviewed one exact result and criteria version;
- Chronos retained the resulting disposition and task state atomically; and
- dependent work became eligible only after human `Accepted` moved the task to
  `Done`.

`RevisionRequired` returns the exact `Succeeded/ReadyForReview` task to
`Ready/Open` only when `attempts < max_attempts`. It preserves attempt history
and does not claim, retry or expand the budget. `Deferred` is a typed held
Maybe with no Chronos mutation. A missing human response is also no decision,
not acceptance, rejection or failure.

The canonical design is
[`chronos-review-disposition.machine.rune`](../runes/chronos-review-disposition.machine.rune).
Its source SHA-256 is
`4dfd4d972512bb853f939784f90a24563487d602aec3b4e481ca4884e2b30766`;
Scry's typed-IR SHA-256 is
`cdbeb0e44462f6a3f45ed16f99e5ca43011eb750ebd6f507acfc8e6f49b3a566`.

## Grounded ensemble evidence

Pantheon's shipped `tellus-sme-plan` command made an authenticated read-only
request to an isolated Dendrite owner through
[`chronos-review-disposition.sme-plan.json`](../requests/chronos-review-disposition.sme-plan.json),
SHA-256
`2eb629b7a0748f5098a067911f1288c59e28fc6b6d8f756b7d53bdeb034fdad8`.
The canonical fold digest was
`14fad3a0af935394c7484e88304d7e2d9636ff76ff7517301808c638b4d31be8`;
the lattice digest was
`2ff1bddf17d36f1e202e68073e9df8298a34b8f8b453a2e6ac1f00758d46d508`.

Architecture, network, identity, data, user-experience and humanism activated
surfaces were returned with exact authority and surface digests. The plan is
honestly `review_required`, not `current`: user-experience and humanism share
the `human-outcomes` facet. The technical/domain facets provide independent
coverage adequate to draft this contract, but the human sponsor, actual review
authority and affected stakeholders retain independent standing. SME routing
does not approve the design.

The live probe exposed one owner DTO defect. Dendrite projected the fold
query's tag-to-role objects where Tellus's typed contract requires the exact
caller tag strings, so Pantheon correctly returned `InvalidResponse`. Dendrite
now projects `request.tags`, and its endpoint regression asserts that exact
shape in Dendrite commit `ddfb355`. Four focused endpoint tests and formatting
pass; the same real Pantheon command then decoded the six-role plan
successfully. No parser or contract was weakened.

## Qualification and path finding

- Smith canonical formatting passes.
- Real Scry admits the eleven-state, fifteen-edge design.
- Smith Rust compatibility is `complete` with an empty unsupported set.
- Qualified Scry SHA-256 is
  `cac27f87a01181d0b0075d752cc95cbecb8597b5091dfc0323a6345151df46b4`;
  Smith SHA-256 is
  `3288271a2dcae7bd833ddfea15a16562c5fb4335b16e0373e34c45bfa998504c`.

The first path check found a semantic defect: one shared `outcome-unknown`
state allowed the graph to forget whether Accepted or RevisionRequired had
possibly reached Chronos. The machine now has separate
`acceptance-outcome-unknown` and `revision-outcome-unknown` states. The accepted
path can no longer bypass `acceptance-invocation-prepared`. The remaining six
path-check failures are the legitimate alternative revision, held and refusal
terminal sinks and the revision-lineage states that cannot reach Accepted;
they were not removed to satisfy a single-terminal checker.

## Required owner implementation

Chronos still needs a closed review command and durable receipt in its atomic
invocation owner, exact command lookup after a lost reply, current execution
and attempt-budget guards, persistence/version migration, authorization and
retained-result disclosure, and RED/USE plus golden-signal observation. The
engine must hold dependency satisfaction until status is `Done`; current
execution success alone must not release dependents. Acceptance must commit the
review receipt, `Done`, revision and dependent reevaluation together.

The headless path should be qualified first with Accepted,
RevisionRequired, Deferred, exhausted budget, stale revision, restart,
unauthorized disclosure, commit failure and lost-reply reconciliation. TUI
parity follows through the same behavior facade. Hermes publication and Oculus
presentation observe committed owner facts; neither becomes the decision or
mutation owner.
