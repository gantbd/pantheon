# Workbench composition program

Workbench is the product composition root for independently useful peers. It
owns product navigation, layout, focus, profile selection, deployment topology
and aggregate lifecycle. It does not merge peer authority, invent a universal
semantic protocol or require every peer to use an identical interaction model.

This directory answers how the workspace becomes one product. Semantic
ownership remains in the root `ENGINE_MAP.md`; exact repository revisions
remain in the root `workspace-compatibility.json`; peer behavior remains in the
peer repository; and cross-engine effects remain in their adapter, federation
or Moira roadmaps.

## Required pattern

A peer is reasonably standalone when users or operators benefit from using it
without the rest of Workbench. Such a peer should provide, as its domain earns
the layers:

1. a public client contract with in-process and/or transport interpretations;
2. peer-owned reusable command guards where effects require exact confirmation,
   current-state fencing, or uncertainty handling;
3. a reusable presentation component that owns only peer-local projection,
   form and armed/disarmed confirmation state;
4. a standalone application shell for terminal/window lifecycle, arguments,
   headless operation and process exit;
5. Workbench attachment through the same client, command guards and presentation component;
6. optional Workbench hosting when production adapter, authority, recovery and
   shutdown composition are explicit.

The invariant is shared; the interaction is not. Vellum may be an editor,
Agora a room/message surface, Chronos a task hub, Moira a reconciliation view,
Daimon an execution inspector, Hermes a ledger view, Aegis an authority
inspector and Grounding a provenance/freshness view. Capability identifiers,
commands, confirmations, projections and failure vocabulary stay peer-owned.

Headless mode is a first-class peer interface, not a hidden TUI invocation. It
uses the same public client, constructs no cursor/focus/widget state and emits a
versioned peer-owned result. Workbench may aggregate those results but cannot
turn partial peer success into aggregate success.

## Composition states

The parity matrix uses these states:

- `absent`: no suitable public boundary exists;
- `planned`: ownership and exit evidence are documented;
- `implemented`: code exists with focused tests;
- `qualified`: real peer/service evidence passes at a named source state;
- `release-qualified`: immutable inputs and the claimed distribution gate pass.

Interactive, headless, attach and host are classified separately. Read-only
qualification never implies mutation qualification. Test authority never
implies production credential provenance.

## Current parity matrix

| Peer | Standalone interactive/headless | Workbench attach | Workbench host | Next boundary |
| --- | --- | --- | --- | --- |
| Agora | qualified read | qualified read | implemented with test-only authority | signal/drain process shell; private-endpoint identity, time, bounded bootstrap, public Aegis connection, exact conversation/scheduled-composition grant provisioning, and durable create/open/residue recovery are implemented owner-side |
| Chronos | qualified read and selected guarded writes | qualified, including the selected guarded-write profile | qualified local owner with exact durable mutation reopen | immutable shared Keystone source and mutations beyond the selected task/scheduling profile |
| Vellum | qualified editor/read | qualified | qualified local owner | newer durable operations and release-qualified platform composition |
| Moira | qualified read/reconciliation, census, evidence and receipt projections | qualified attach with the same component and census | absent | guarded reconciliation and native host factory |
| Daimon | qualified redacted presence | qualified attach | absent | settle active engine work, then exact execution evidence and guarded dispatch/resume |
| Hermes | qualified content-free observation/cursor plus separate exact-confirmed bounded event disclosure | qualified attach and the same disclosure guard | qualified owner process in both modes | guarded producer-specific append |
| Aegis | qualified read-only authority observation, bounded pages, exact-confirmed epoch revocation and named managed authority session | qualified observation/navigation plus the same owner guards, managed session, disconnect abandonment and deliberate rebuild | qualified owner process in both modes | no selected-profile parity gap; durable replay/reconciliation are nonclaims |
| Grounding/Dendrite | qualified typed observation | qualified typed attach | absent | selected surface/read DTOs and deployment-owned hosting |

The detailed open-boundary ledger remains `WORKSPACE_GAPS.md`. The ordered
delivery sequence is in `backlog.md`; Axon and Engram adoption decisions are in
`prior-art.md`. The checked cell classification is
[`composition-status.json`](../../composition-status.json); prose must not claim
a stronger state than that file.

## On-disk routing

| Question | Canonical location |
| --- | --- |
| Which repository owns a semantic concern? | root `ENGINE_MAP.md` |
| Which exact repository states form current evidence? | root `workspace-compatibility.json` |
| What is the common component/presentation law? | `COMPONENT_ARCHITECTURE.md` |
| How does the product compose peers? | this directory |
| What does one peer do standalone? | that peer's README and roadmap |
| What crosses two engines? | the owning adapter/federation roadmap or Moira saga |
| What did Axon/Engram teach us? | `prior-art.md` |
| What is next? | `backlog.md`, linked to the owning detailed roadmap |

This prevents the root from becoming a product backlog, Workbench from becoming
a semantic engine and Keystone from becoming a presentation or integration
dumping ground.
