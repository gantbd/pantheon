# Rune typed child invocation and return request

Date: 2026-11-20

Status: owner request; not an admitted Rune change, compiler design decision,
implementation, compatibility claim, Chronos dependency or rollout authority

## Driving intent

Rune must express one bounded child-machine computation and return its typed
evidence into one exact parent obligation without semantic loss. This is the
load-bearing substrate for Pantheon's center-out recursion and the parent Maybe
Machine. A caller-supplied Boolean stating that a named child halted acceptingly
is not sufficient: it erases verdict, provenance, bounds, uncertainty, refusal,
budget and the exact parent obligation the result may satisfy.

## Current grounded boundary

The inspected Rune peer checkout reports commit
`22313327fcff68a02ddcfa18b8464dae3d3c03db` with extensive existing modified and
untracked work. This request does not mutate or claim that checkout.

Relevant inspected byte identities are:

- Scry machine parser, `src/ocaml/scry.ml`:
  `44e5c61eca715512974388220401a3389e22c4778ab394033c3b1e17124c465b`;
- shared typed IR, `src/rust/shared/src/ir.rs`:
  `c217a7e925eb7c6a305c57aa2d211cd5be1506375a0e5a709cdc23610128fe15`;
- Smith compatibility analyzer:
  `529269483238e94a47ea442e0f59d93263b223188fe6182d6208d8d1aea18084`;
- Gleam code generator:
  `aa2daf6db516300aa6365c55e6595a41bfc314f5e676abc00ba78bdf2208b322`;
  and
- Pantheon's exact rejected recursive candidate:
  `7355979f495fe75aba57e41e2fe89208da97c899ccb12bffe8ff7537cd0f7c69`.

Scry's closed machine-edge grammar does not admit `(invoke …)`. The lower Rust
IR nevertheless has one optional string-valued `invoke`, and Gleam lowers it to
`invoked: fn(String) -> Bool`. That is useful historical direction but cannot
carry a child result or establish which exact child bytes, input, execution,
budget or evidence produced `True`.

## Rosetta

- **Intent:** admit a canonical, closed, bounded child-invocation construct whose
  return is a typed evidence carrier consumed by one exact parent edge.
- **Values:** semantic preservation, singular composition, provenance,
  uncertainty honesty, refusal monotonicity, finite recursion, owner boundaries,
  target parity and fail-closed projection.
- **Outcomes:**
  1. canonical Rune source names the exact child machine/version constraint,
     input carrier, return carrier and parent acceptance predicate;
  2. Scry validates source shape, type compatibility, singular binding,
     well-founded budget and cycle policy, then preserves them in typed IR;
  3. a child return retains verdict, diagnosis, bounds, provenance, evidence
     references, dissent, uncertainty/refusal, consumed budget and exact child
     identity;
  4. the parent advances only when exactly one returned envelope satisfies its
     typed predicate; zero, multiple, mismatched, unavailable, uncertain or
     refused results remain distinguishable and cannot be repaired implicitly;
  5. Smith compatibility and emission refuse every target that cannot enforce
     the complete contract; and
  6. generated execution grants no child effect or authority by itself.
- **Who:** Rune language/compiler owners, Smith target owners, machine authors,
  runtime adapters, independent verifiers and consumers such as Pantheon/Moira.
- **What:** one language and typed-IR extension plus target compatibility law;
  exact surface syntax remains an owner design decision.
- **When:** before a parent Rune claims recursive composition, monadic evidence
  folding or operational Maybe Machine semantics.
- **Where:** canonical machine dialect, Scry typed IR, Smith compatibility and
  executable targets; runtime owner adapters remain separate.
- **Why:** aggregating task success or accepting an opaque Boolean child halt
  cannot prove preservation of the parent obligation.
- **How:** design the carrier and invocation relation first; admit closed syntax;
  validate and canonicalize; add compatibility enforcement; then implement one
  target at a time behind refusal for all others.
- **Extent:** pure orchestration/evidence semantics only. No provider call,
  filesystem/shell access, effect execution, owner receipt invention, human
  disposition, Bool promotion, task mutation or rollout authority.
- **Example:** a Maybe inquiry invokes one bounded owner-observation child with
  exact claim and remaining fuel; the child returns `Unavailable` with source
  identity and retry prohibition; the parent remains maybe and cannot take the
  evidence-satisfied edge.

## Required semantic decisions

The Rune owner must decide and document, rather than leave to target wrappers:

1. how canonical source identifies an exact child definition or compatible
   version without ambient path lookup;
2. how input and return carrier types are declared and checked;
3. how one child invocation identity binds parent instance, edge, attempt and
   replay without becoming effect authority;
4. how shared finite fuel decreases across nesting and how exhaustion returns;
5. which recursion/cycle forms are forbidden, statically bounded or dynamically
   budgeted;
6. how child refusal, uncertainty, held plurality and outcome-unknown remain
   distinct and monotone through reassociation;
7. how a parent predicate consumes the returned carrier without a free-form
   callback laundering unsupported semantics;
8. how restart/replay observes an existing child result rather than duplicating
   work; and
9. which exact laws make nested invocation reassociate without changing meaning,
   authority, budget, evidence identity or refusal.

## Mandatory negative evidence

Qualification must refuse at least:

- unknown, ambiguous, mutable or incompatible child identity;
- missing or duplicate invocation binding;
- input or return carrier mismatch;
- missing, duplicate, contradictory or forged child return;
- a Boolean accepting-halt value presented without its typed envelope;
- parent advancement from child refusal, uncertainty, truncation,
  outcome-unknown or budget exhaustion unless the exact edge declares that
  tagged result as its input;
- unbounded recursion, an unadmitted cycle, fuel increase or per-child fuel
  reset;
- replay under a different parent, edge, child version, input or authority;
- a target claiming compatibility while dropping any carrier field or law; and
- code emission after compatibility refusal.

## Minimum proof and qualification matrix

1. Canonical formatter round-trip preserves the complete construct.
2. Scry emits reproducible typed IR and rejects every malformed/unknown clause.
3. Type and graph checks cover child existence, carrier compatibility, cycles,
   fuel and exact return-to-edge eligibility.
4. `machine_path_check` accounts for invoked paths and the parent's closed set
   of tagged return outcomes rather than one Boolean accepting halt.
5. Smith's single compatibility analyzer both reports and enforces support.
6. Rust either implements the full contract with target-verified generated
   tests or refuses `typed-child-invocation-return`; no structural approximation
   is emitted.
7. Existing Gleam support is upgraded from `fn(String) -> Bool` to the same typed
   carrier or is reclassified unsupported until it is.
8. At least one bounded parent/child fixture proves accepted return, refusal,
   uncertainty, fuel exhaustion, replay, wrong-child/wrong-parent rejection and
   nested budget preservation.

## Nonclaims and handoff

This request does not prescribe syntax, choose Moira as runtime host, prove
monad laws, make the Maybe Machine executable or authorize editing the dirty
Rune checkout. The Rune owner should first reconcile this request with its
current uncommitted work and issue an attributable design disposition. Pantheon
must continue to retain the invocation gap until an exact admitted source,
typed-IR and target qualification exist.
