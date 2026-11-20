# J1 UADD composition authoring questions

Date: 2026-11-20

Status: `questions_pending`; zero candidate, typed IR, projection, owner effect,
human decision, acceptance, task disposition or rollout authority

## Bound request

- structured authoring request:
  `design/requests/j1-uadd-composition.rune-author.json`
- request SHA-256:
  `c3036260331925cb89cc5104bbd33c42230b397f9cc67223751210cbe5db2e29`
- governing owner request:
  `design/requests/j1-uadd-acceptance.rosetta.md`
- canonical method prior art:
  `rune/fsm/user-acceptance-driven-development.machine.rune`

The authoring request retains the exact UADD-to-J1 intent, outcomes, actors,
scope, nonclaims and examples. It does not include an accountable review or a
machine topology because neither has been admitted.

## Deterministic result

The existing Rune-owned authoring boundary was invoked without arguments and
received the exact JSON request on standard input:

```text
../rune/src/rust/target/debug/rune-author < \
  design/requests/j1-uadd-composition.rune-author.json
```

It returned version 1, status `questions_pending`, machine state
`questions_pending`, signal `clarification-required`, and the same request
digest prefixed with `sha256:`. It emitted no candidate, typed IR or Rust
projection. This is the required outcome while reviewed semantics remain open,
not a compiler failure.

The result retained fifteen typed questions:

- ten exact owner-contract holes covering request ownership, challenged-material
  identity, challenge result semantics, bounded STRIDE evidence, finding
  disposition, delegation policy, UADD response typing, Chronos mapping,
  lost-reply reconciliation and content-safe observability;
- one authority question asking who reviewed the exact request and what the
  ensemble-fit answer is; and
- four topology questions for the initial state, states, transitions and
  emitted signals.

The tool restated its least-authority boundary: deterministic construction,
Scry validation and private target-verified Smith projection only, with no model
inference, workspace or task mutation, approval, publication or rollout.

## Admission boundary

An authenticated Tellus/Dendrite ambient-expertise review and a separately
equitable Moira stakeholder/owner ensemble must review this exact request and
answer “do we have the right ensemble?”. The named human design authority must
then review the exact revised request. Relevant owners must answer the ten
contract questions before topology is treated as law. Any resolution produces
a new request digest; the current digest must not be silently reinterpreted.

The first authenticated Tellus attempt is now bound separately in
`design/evidence/j1-uadd-sme-review-refusal.md`: transport and authentication
passed, but the real Axon lattice refused the selected adversarial-tester
because its Rune grounding has no concern registration. That owner gap must be
resolved without dropping the adversarial review before an SME plan exists.

Only then may `rune-author` be rerun. A candidate is review material, not an
admitted machine. Exact Scry validation, complete Smith compatibility, private
target verification, authorized revision-bound Vellum publication and
independent qualification remain subsequent gates.
