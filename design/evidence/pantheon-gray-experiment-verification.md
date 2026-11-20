# Pantheon Maybe Machine bounded-experiment leaf verification

Date: 2026-11-20

Status: structurally admitted child design candidate; no parent Maybe Machine,
experiment authority or runtime qualification

## Bound artifacts

- governing-intent candidate SHA-256 bound when this artifact was admitted
  (now a predecessor):
  `0d8872543501cca6b776c30fc3b8b18892590c3cf79e161118c4889f2382fb26`
- canonical Gray Rune:
  `design/runes/pantheon-gray-experiment.machine.rune`
- canonical Rune source SHA-256:
  `e76506d6e8c314a327cf2746627c7116999abaa8f10bc3cb453b31ef586a98ca`
- typed-IR SHA-256:
  `bdf14ce25f811b4d3247568a43280da1d49169a1497c78e8e761acb87bf057be`
- Smith SHA-256:
  `3288271a2dcae7bd833ddfea15a16562c5fb4335b16e0373e34c45bfa998504c`
- exact private Rust projection SHA-256:
  `d2fce8230a045d2c4e5e6ca8af3c104ad01350345f5dcdbb809ed85a3f713962`

The `gray-experiment` filename above is a provenance-bound historical identity,
not the governing ontology. The Gray is the whole Maybe Machine; this artifact
is only its bounded-experiment child. A successor revision should use a
child-specific identity such as `pantheon-maybe-bounded-experiment` while
retaining this predecessor and its hashes rather than rewriting them.

## Preserved design

The machine is one bounded-experiment leaf of the larger Maybe Machine. It does
not select experiment over deterministic proof, owner observation, stakeholder
deliberation, human return, held plurality or refusal, and it cannot admit its
own output into Bool. That parent selection and return contract is not yet
executable.

Within its declared leaf scope, the machine retains one exact question, makes applicable controls reviewable,
requires a bounded experiment plan and exact owner/human authority before any
operation, returns evidence before interpretation, and distinguishes supported,
refuted, unsettled and safely-refused results. The experiment plan must name its
current and target altitude, nearest owner-observable fact, cheapest adequate
grounding descent, scope, capabilities, data, stakeholders, cost, budget,
duration, isolation, reversibility, oracle, observability, stop conditions,
prohibited effects and nonclaims.

The Rune operates inside maybe. Running an experiment does not make its hypothesis Bool,
authorize rollout, modify a control or prove stakeholder acceptance. Missing or
lost owner replies remain uncertain and grant no retry authority.

## Semantic correction during review

The first structurally valid draft treated `control-revision-required` as a
terminal alternative to `proposal-supported`, `proposal-refuted` and
`proposal-unsettled`. That was semantically wrong: experimental interpretation
and control disposition are orthogonal. One run may both support a hypothesis
and show that a control needs revision.

The corrected machine retains any control finding in the evidence packet while
leaving the current control unchanged. A separate owner-authorized control
supersession machine must consume that finding. This prevents both silent bypass
and a false exclusive choice introduced merely to fit an FSM.

## Tool evidence and limits

Scry admits the ten-state, nine-edge Rune. Smith reports complete Rust
compatibility with an empty unsupported set, and the exact generated projection
compiles as a Rust 2021 library.

`machine_path_check` cannot qualify the design's four legitimate terminal sinks.
When asked to prove the `proposal-supported` terminal with the six common stages,
it returned exit code `1`, reporting the other three sinks and four failures.
The Rune was not collapsed into a generic terminal, because generated state
alone would then lose supported, refuted, unsettled and safely-refused meaning.
The existing multi-terminal qualifier gap therefore remains open.

Compatibility and compilation prove supported projection shape only. They do not
prove that:

- the selected ensemble has the right affected stakeholders or independent
  control reviewer;
- any control is valid or invalid;
- any experiment is adequately contained, affordable or authorized;
- an oracle measures the intended property;
- the generated caller refuses zero or multiple eligible branches;
- any owner effect, evidence return, control revision, acceptance or rollout
  occurred; or
- the governing-intent candidate has been ratified.

## State-local Tellus grounding probe

A bounded live probe exercised Pantheon's real `tellus.sme-plan.v1` client
against an isolated authenticated Dendrite instance on `127.0.0.1:4501`. The
already-running service on port `4499` was not replaced or mutated: its resource
metadata answered, while its Tellus observation route returned HTTP 500. The
isolated instance was therefore a reversible diagnostic, not evidence about the
deployment owner's service health.

The first request incorrectly named `rune/sme/orthogonality` and was refused.
Read-only corpus discovery corrected the lattice identity to
`axon/sme/axon-concerns`. An all-machine tag union then routed ten roles and was
correctly refused by Tellus's eight-role ceiling. The ceiling was not raised.
The request was instead descended to the first actual decision point,
`control-challenged`, using only:

- `access-control-decision`;
- `architecture-flow-review`; and
- `human-impact-decision`.

The exact fold digest was
`14fad3a0af935394c7484e88304d7e2d9636ff76ff7517301808c638b4d31be8`.
It routed identity, security, architecture, network, data and humanism. All six
activated surfaces resolved with exact authority and surface digests, so SME
content availability was not the failing boundary. The subsequent exact
orthogonality query refused with `role_missing_concern`: `rune:security` has no
authored `(concern …)` clause and is therefore not registered in the lattice.
Pantheon's SME plan consequently remained `Refused(503)`. No correlated or
independent ensemble claim was inferred and the lattice was not bypassed.

This probe lowers the unresolved claim from “Tellus planning is unavailable” to
one owner-observable corpus fact. The nearest adequate descent is now an
owner-authored security concern registration (or an explicit owner decision
that the role cannot participate in orthogonality), followed by the same exact
state-local request. Separately, Tellus observation returned a maximum of 64
alphabetically ordered entries with `truncated: true` and no continuation
cursor. Its `complete` flag describes freshness of that selected page, not
complete inventory discovery; that surface gap must remain visible.

The six routed roles are advisory SME coverage only. They do not answer “do we
have the right ensemble?” and do not provide equitable stakeholder standing.
The experiment owner, control owner, operator, and affected experiment subjects
still require independently attributable Moira participation before any
consequential Gray experiment can be admitted.

## Next admission boundaries

1. Add or explicitly refuse the missing security concern registration, rerun
   the exact state-local Tellus plan, then admit an equitable stakeholder
   ensemble through Moira, beginning with “do we have the right ensemble?”.
2. Define a typed experiment-plan and evidence envelope whose altitude and
   grounding-descent fields are owner-verifiable rather than prose alone.
3. Supply a strict evaluator that refuses zero or multiple result branches.
4. Author the separate control-supersession machine, preserving predecessor,
   protected invariant, evidence, affected standing and human/owner gates.
5. Extend Rune path qualification to an exact closed terminal set before making
   mandatory-stage claims.
6. Exercise one non-production, reversible real-owner experiment and deliberately
   validate the instrument's failure behavior before trusting its result.
