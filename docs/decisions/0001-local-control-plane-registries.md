# ADR 0001: Reuse sovereign participation and work-mode owners

Status: accepted for incremental composition

Date: 2026-11-17

## Context

Human/agent participation and custom work-mode definitions need durable,
versioned truth. Keeping either in presentation loses state and creates hidden
authority. An initial proposal placed both in a new `pantheon-control` process.
A repository-wide reuse audit then found that Moira already publishes a durable,
revisioned, restart-safe governed-agent participation service and reusable TUI
component. Rune, Dendrite and Moira already divide authored methods,
distribution/grounding and workflow instantiation.

Creating a parallel registry before composing these contracts would duplicate
truth and violate Pantheon's role as a presentation and journey host.

## Decision

Do not create `pantheon-control` for the current milestones.

For J1 agent participation, compose Moira's existing public
`agent_participation` protocol, Unix client and Ratatui component. Pantheon may
select and project one exact participation identifier but does not copy its
record, infer authority from its role/persona or host its storage.

For broader human/freeform participation, define a shared Keystone projection
law only after at least two sovereign owner records demonstrate the common
fields. Agora, Moira, Chronos and other semantic owners retain their native
records. A universal registry requires a later ADR proving that continuity
cannot be composed without creating a new owner.

For custom work modes, first compose Rune-authored definitions, Dendrite-pinned
distribution/grounding and Moira workflow or deliberation instantiation. Select
a new lifecycle owner only for requirements those contracts demonstrably cannot
meet; do not put that owner inside Pantheon by default.

The local trusted-development profile claims replayable application history,
not hostile-write resistance. Aegis remains the authority-decision source, and
a role or work-mode label grants no authority.

## Consequences

- Pantheon remains a client and projection host, not a semantic control plane.
- J1 can advance immediately by reusing Moira's qualified participation owner.
- Human/agent parity remains a product requirement but not a pretext to flatten
  different owner semantics into one premature store.
- J34 begins with an owner-capability audit across Rune, Dendrite and Moira.
- Any new service must name the missing invariant, two real consumers, recovery
  behavior and why extension/composition of an existing owner is insufficient.
- The abandoned provisional `pantheon-control` crate created during exploration
  is not part of the product or workspace.
