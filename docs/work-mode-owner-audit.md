# Declarative work-mode owner audit

Status: capability audit complete; sovereign lifecycle owner selection requires
a human architecture decision

Governing milestone: P1 in
[`journey-execution-plan.md`](journey-execution-plan.md)

## Outcome

Rune, Dendrite and Moira provide three necessary but deliberately different
parts of a declarative work-mode journey. None currently owns the complete,
durable definition lifecycle required by acceptance cases 98–105. Pantheon must
not fill that gap with a private registry: it is the projection and action host,
not the semantic owner.

The smallest safe addition is a separately deployed, revisioned work-mode
definition registry in Rune's ecosystem layer. That recommendation does not
authorize implementation or select a repository. A named human architecture
owner must decide whether to create that service or admit an existing sovereign
owner after reviewing the cross-project boundary, operational cost and migration
plan.

## Observed capability matrix

| Capability | Rune language/tooling | Dendrite | Moira | Pantheon |
|---|---|---|---|---|
| Author and validate a definition | Yes: `.rune` language, corpus and owned validators/tooling | Consumes validated Rune content | Retains exact method/grounding digests in instances | Presents authored/validation results |
| Durable definition registration | No service contract found | No; `load` registers a query session/corpus, not a product definition | No; `Admit` creates a deliberation or participation instance | Must not invent one |
| Exact definition resolution | Tools resolve and canonicalize authored content | Resolves loaded references and surfaces within a `load_id` | Pins opaque method and grounding digests in an instance | Can compose exact owner projections |
| Activate for new use | No runtime lifecycle owner | `load`/reload controls query availability, not work-mode activation or authority | Instantiates a selected deliberation/participation | Can request an admitted owner transition |
| Migrate definition or instances | No generic lifecycle contract found | Hot reload re-resolves content and preserves the prior index on failure; this is not version migration | Instance owners can evolve only through their explicit protocols; no generic work-mode migration exists | Can present a migration plan and receipts |
| Deprecate new use | No generic work-mode status contract found | No work-mode deprecation contract | No definition deprecation contract | Can project status and route an authorized request |
| Retire | No generic work-mode lifecycle contract found | `unload` removes a query session, not a durable definition or its evidence | Individual instances have terminal behavior, not definition retirement | Can project status and route an authorized request |
| Tombstone | No generic retained-marker contract found | No definition tombstone | No definition tombstone | Must not delete or relabel owner evidence |

Important nonclaims:

- Rune files are authored inputs, not durable registry receipts.
- A Dendrite `load_id`, hot reload or unload is not a work-mode activation,
  migration, deprecation or authority decision.
- A Moira persona lens is not a role/work mode, and a deliberation admission is
  not definition registration.
- Pantheon displaying or invoking a transition does not make Pantheon its owner.
- Rune Ley/ADR lifecycle statuses are not evidence of a generic work-mode
  lifecycle contract.

## Missing owner contract

The selected owner needs a bounded public contract with:

- immutable definition identity, semantic version/major identity, canonical
  digest and declared compatibility class;
- exact authored Rune reference/digest and optional Dendrite surface identities,
  without embedding executable content, bearer material or hidden routes;
- revisioned states for admitted/active-for-new-use, deprecated, retired and
  tombstoned-as-a-retained-marker;
- stable command identity, expected revision, exact replay/conflict behavior,
  named actor/principal and separate Aegis authorization evidence;
- register, exact resolve, compatibility assessment, deprecate, retire and
  tombstone commands with durable receipts;
- explicit migration-plan records that reference affected owner-native
  instances without mutating those instances directly;
- bounded list/query projections, prior/current revision recovery and reasoned
  refusal/uncertainty results;
- accountable human owner, rationale, policy epoch, effective/expiry time where
  policy requires them; and
- retention rules that never turn tombstoning into destruction or rewriting of
  prior decisions and instance evidence.

Instance activation and migration remain with the instance owner—initially
Moira for methodical deliberations/agent participation. Dendrite continues to
own content loading and exact grounding projections. Aegis decides authority.
Pantheon composes the user journey and never broadcasts lifecycle events as if
that made it authoritative.

## First vertical slices

### Slice A — register and resolve `BusinessReviewV1`

1. Author and validate the bounded declarative definition through Rune tooling.
2. Register its exact version/digest with the selected definition owner under an
   expected revision and authorized human sponsor.
3. Resolve the exact version through TUI and headless paths; unknown, ambiguous,
   retired and digest-mismatched requests refuse without nearest-name fallback.
4. Instantiate one Moira-owned review pinned to that definition and exact
   Dendrite grounding surfaces.
5. Reopen all owners and prove the same definition/instance identities and
   receipts.
6. Deprecate only new use while the pinned review remains observable and able to
   complete; do not implement migration or tombstone in this slice.

### Slice B — coexist with `ProjectProposalV1`

1. Register and resolve the proposal mode independently with an accountable
   Executive Sponsor requirement.
2. Prove both definitions coexist and each instance remains pinned to its exact
   version/digest.
3. Present a compatible additive revision as new-use-only until an explicit
   owner migration plan is approved; preserve old instances unchanged.
4. Treat a breaking semantic revision as a distinct major identity.
5. Qualify human go/no-go at proposal stages and prove initial sponsorship cannot
   be replayed as final rollout approval.

## User-advocacy and configuration requirements

- Catalogue, register, resolve, instantiate, deprecate and recover must be
  discoverable UI journeys. Ordinary users must not edit JSON, copy digests or
  invoke repository commands.
- The UI must explain the difference between authored, loaded, active-for-new-
  use, instantiated, deprecated, retired and tombstoned states.
- Workspace defaults may select catalogue filters, grounding depth, notification
  thresholds and approved provider profiles, but semantic identity, authority
  and human gates cannot be weakened by a display preference.
- Every refusal names the responsible owner, expected/actual revision, affected
  definition/instance, preserved state and next safe action.
- Configuration changes need preflight, redacted typed diff, impact scope,
  restart boundary, receipt and rollback/recovery orientation.
- The Chronos operator agenda should surface pending registrations, migration
  reviews, deprecations, retirements and tombstones with deadline, consequence,
  evidence, dissent and exact authority; appearing in the agenda grants none.

## Local-Qwen delegated review disposition

A bounded Qwen3 Coder Next turn through the already-qualified local LM Studio
profile reviewed the evidence packet as a product-owner/user-advocate lens. It
was advisory and had no tools or authority.

Useful findings retained:

- no current component owns the generic durable lifecycle;
- Dendrite load state must not be presented as work-mode activation; and
- lifecycle decisions need receipts, visible configuration and human review.

Recommendations rejected after source verification:

- assigning lifecycle ownership to Pantheon (violates the projection-host
  boundary);
- counting authored Rune files as durable registration;
- inventing Pantheon-to-peer lifecycle events and Moira migration behavior;
- claiming Dendrite hot reload bypasses validation (it re-runs `rune_loader` and
  preserves the prior index on failure); and
- prescribing global role-reactivation flags or stakeholder lists without a
  selected owner/policy contract.

This review is evidence that persona prompting can produce useful questions but
cannot substitute for owner inspection, human advocacy or accountable decisions.

## Human decision gate

Before implementation, a named human architecture sponsor must approve one of:

1. a new independently deployed Rune-ecosystem definition registry (recommended
   for the cleanest boundary);
2. an explicitly expanded existing sovereign service, with evidence that the
   added lifecycle belongs to its purpose; or
3. deferral with a documented inability to deliver P1/J34–J36.

Extending Pantheon itself is a no-go because it would make the presentation host
the semantic owner. Extending Dendrite or Moira by convenience alone is also a
no-go. Final production rollout remains a separate named-human go/no-go even
after the owner decision and slice qualification succeed.
