# Governing-intent admission machine verification

Date: 2026-11-20

Status: structurally admitted candidate; runtime admission has not started

## Bound candidate

- governing-intent candidate:
  `docs/governing-intent.md`
- candidate SHA-256:
  `65582e3c16e2da06431e99395d3103d7cb01457b77ab1c84c0704a552072920a`
- immediate predecessor candidate SHA-256 (superseded when the Gray/Maybe
  naming law was made explicit):
  `0d8872543501cca6b776c30fc3b8b18892590c3cf79e161118c4889f2382fb26`
- earlier predecessor candidate SHA-256 (superseded when The Gray was
  recognized as the Maybe Machine):
  `a26b1a33b0e91f99b82c8b72c450f5e8c39f212644a445de1ff739431f4aff1a`
- original predecessor candidate SHA-256 (superseded after archaeology):
  `d1785547c23e6b19fd7687726a7ee38c79014bac908a9f69e75a63a73644c05a`
- archaeology evidence SHA-256:
  `eb51331a460801bf51da6dafa4a0d44dc5d5cfb7858852f0e018a8c7f6ab0d1b`
- admission Rune:
  `design/runes/pantheon-governing-intent-admission.machine.rune`
- canonical Rune source SHA-256:
  `78f963bb2d803446cff96e5b7a4496ace3bba2ceab8e7e821bbd23c8f9f8d4c7`
- typed-IR SHA-256:
  `042f146be87a00fd0f4af0ce9ada1e1d173d605403f01e3dd9cec9a211e8a1bd`
- Smith SHA-256:
  `3288271a2dcae7bd833ddfea15a16562c5fb4335b16e0373e34c45bfa998504c`
- private target-verified Rust projection SHA-256:
  `af44d3a2556d7bc40577fa6604f2251571014cb9b49df1c29b9939314f321fff`

## What the tools prove

Scry admits the seven-state, six-edge machine. The candidate bytes were revised
after Pies and Engram archaeology; this packet now binds the revised digest and
retains the predecessor rather than treating an in-place edit as acceptance.
The later Maybe Machine correction and the subsequent proper-name/child-name
correction each produced a new candidate identity. Both are retained as
explicit predecessor transitions rather than in-place acceptance claims. The
Rune's examples refer to the exact candidate identity generically;
candidate bytes and digest remain evidence inputs rather than mutable literals
inside the reusable admission machine. The canonical formatter check and Scry
admission were rerun. Smith reports complete Rust compatibility with an empty
unsupported set; the exact private Rust projection compiles as a Rust 2021
library. The machine keeps candidate
ownership, ensemble review, pending human disposition, acceptance, refusal,
revision request and immutable issuance semantically separate. Its chooser is
pure evidence control; it performs no Vellum, Moira, Agora, Aegis or other owner
effect.

These results prove submitted structure and supported projection only. They do
not prove:

- that the candidate has a Vellum or other sovereign owner receipt;
- that the right stakeholder ensemble has been admitted or reviewed it;
- that the human sponsor accepted, refused or requested revision;
- that `GoverningIntentV1` exists;
- that the generated chooser has a qualified strict typed-evidence wrapper; or
- that any downstream task, implementation, acceptance or rollout is aligned.

The current runtime state is therefore **before `candidate-owned`**. The
repository candidate digest is provenance, not the state's required owner
receipt. Conversational agreement and “Namaste” were deliberately not converted
into acceptance of exact bytes.

## Mandatory-path qualifier gap

`machine_path_check` currently requires one nominated terminal and rejects a
graph whose actual terminal-sink set contains any other state. This machine has
three intentionally disjoint terminal outcomes:

- `intent-issued`;
- `candidate-refused`; and
- `revision-required`.

Each attempted per-terminal proof returned exit code `1` because the other two
legitimate sinks exist. The machine was not weakened into a generic terminal:
Smith's generated state alone would not preserve a tagged disposition payload
after such a convergence. Scry's admission establishes the closed reachable
graph, but this packet makes no mandatory-path dominance claim.

The smallest honest qualifier follow-up is a Rune path-proof extension that can
accept an exact closed set of terminal variants and prove:

1. every declared state is reachable from the initial state;
2. every state can reach at least one admitted terminal variant;
3. each terminal has no outgoing edge;
4. required common stages dominate the terminal set collectively; and
5. variant-specific stages dominate only their declared terminal subset.

Until that exists, the Scry and Smith evidence remains useful but incomplete
for a full algebraic admission proof.

## Current Vellum owner-surface finding

The repository's current `pantheon agent-tools` projection was executed after
the archaeology revision. It exposes `vellum-document-read` as a default-denied
`workspace.snapshot.v1` operation and the CLI separately exposes revision-bound
durable-save execute/lookup. Neither surface is an immutable governing-candidate
owner:

- a snapshot proves a bounded observation of one admitted locator, not candidate
  freeze, predecessor lineage, review status or authority transition;
- a durable-save receipt proves one owner save operation, not that the saved
  bytes have become an immutable IV&V candidate; and
- Pantheon's machine-readable unavailable catalogue explicitly reports
  `vellum / workspace.changeset.propose.v1` as `surface_not_admissible`, with no
  aggregate fallback. Its missing contract includes revision-bound proposal,
  diff, conflict handling, validation, atomic publication, uncertain-write
  lookup, candidate freeze, authority transition and owner receipts.

Starting `make run`, directly editing through hosted Vellum or relabeling an
ordinary save receipt would therefore weaken the admission machine. Existing
Chronos work for revision-bound Vellum diff/change-set behavior, exact document
observation, durable identity and the bounded frontier capability sandbox must
be reconciled and dependency-linked; this packet does not create a duplicate or
claim that any one partial item already owns the complete change-set contract.

This is an owner-surface gap, not authorization to substitute the repository
digest. The runtime admission state remains before `candidate-owned`.

## Next owner-bound steps

1. Configure and use Vellum or select another explicit sovereign owner to issue
   an immutable candidate receipt over the exact bytes.
2. Admit the stakeholder ensemble, beginning with “do we have the right
   ensemble?”, and retain independent reports, standing, dissent and gaps.
3. Present the exact owner-bound candidate and review packet to the named human
   sponsor through a real human-gate owner.
4. Follow only the exact accepted, refused or revision-required edge.
5. Issue `GoverningIntentV1` only after an exact affirmative receipt.

No later delivery-machine or Chronos projection should claim governing-intent
alignment before these steps are true.
