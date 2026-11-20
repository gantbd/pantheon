# Moira actor-participation relationship verification

Date: 2026-11-20

Status: owner relationship, durable V7 retention, private consuming adapter and
public read-only observation qualified; Chronos scope/task-conversation
composition and J1 review-link composition remain open

## Preserving design

The canonical Rune
`design/runes/moira-actor-participation-relationship.machine.rune` retains one
exact Agora Host-derived actor and subject publication beside one exact
already-admitted Moira participation. Equal subject and admission-principal
bytes are necessary but insufficient. The domain also requires the retained
participation blueprint, admission attribution, task attempt and exact room
source/sequence before insertion.

- Rune SHA-256:
  `0e9671006450d01393b7efa864ddb2cf7516a03fcb193c8955043c469b5e1fa9`;
- Scry typed-IR SHA-256:
  `26483a11feae4fd5f2216d0ac3a213723b97ca1edb02152a00e16e7e86583d92`.

Real Scry admitted the four-state, three-edge design. Smith canonical formatting
passes. Smith's Rust compatibility report is `complete` with an empty
`unsupported` set, including transition preconditions, destination
postconditions, rung enforcement and executable source/IR provenance. This is
design compatibility evidence, not a generated-runtime claim.

## Qualified owner implementation

Moira revision `3fd35de` adds an independent
`ActorParticipationRelationshipV1` rather than changing the existing journey,
room, namespace or participation V1 records. Its complete evidence includes:

- exact journey and journey revision;
- complete retained participation binding, including workflow, deliberation,
  member, Daimon session/profile, task attempt, method, grounding and persona;
- participation-owner revision and exact admission authority attribution;
- Agora effect, source, room sequence and room ID;
- message sequence and message identity;
- Agora revision, Host-derived actor and verified subject.

The first bind requires `ParticipationAdmitted`, the exact retained blueprint
and attribution, task-attempt equality, room source/sequence equality and
subject equality with the exact admission principal. Exact replay leaves the
storage revision unchanged. Every changed publication conflicts. Actor, room,
subject, participation and authority material remain redacted from diagnostics.

Checkpoint `MOIRATJ1` V7 appends a separately versioned relationship
collection. Versions 1 through 6 remain readable and upgrade with the
relationship absent; no actor, subject or agent relationship is invented.
Canonical close/reopen returns byte-identical V7 state.

The public task-agent contract adds only a journey-scoped read operation.
Protocol DTO, authorization-first service, canonical bounded wire,
capability-shaped client and real Unix daemon/client route are qualified. The
public protocol has no relationship mutation command. The only component
consumer added by this slice is `moira-agora`, which maps the exact
`ActorPublishedSubjectProvenanceV1` observation into the private store mutation.

Source evidence:

- domain:
  `sha256:d5ba56a7c5936d345cb2dcc97e0ff7a206d7661dcc3923bc17e521a3edbd0fb0`;
- checkpoint codec:
  `sha256:83f1de4cd659d5a854b42149fd5f2d6d1ed154d191bb5b0d5b530354f30b24f0`;
- public protocol:
  `sha256:1e020e68ce93ced62366c3efe1b1f0e782435e4305dbffbaf6d9875ab268dbd8`;
- host/store port:
  `sha256:b372a7e98a686c0b272ffb120f38fb1ea71b09693e4e965d037bc2d0000f740b`;
- SQLite adapter:
  `sha256:904076426c4c264bccf9ecf31a51ec8f1a8adc5b8e5e9e727df1b067c7bde371`;
- service:
  `sha256:28fbc08ee6269639c774a68ebde3ef60f481388cb6947ec8faeff09e72e980ca`;
- canonical wire:
  `sha256:c77461de9292f0a1f832f21d33da780bf03ead7442e74da56fecd89a7b36789a`;
- client facade:
  `sha256:a6a75e077b9e926b423a88044497d2a55a48a14dafa4f191229048a03b38dc34`;
- qualified Agora consumer:
  `sha256:7cddd0b95a421c659767c62bca85d12f24ae3ed6fa5cbaf22d3ce2d0f258e073`.

Qualification passed:

- 19 task-agent domain behavior tests;
- 5 task-agent checkpoint tests, including populated V7 close/reopen and
  V1/V2 canonical upgrade;
- complete all-target tests for `moira-core`, `moira-agora`, `moira-wire`,
  `moira-client`, `moira-service` and `moira-sqlite`;
- the real local Unix daemon/client task-agent test outside the sandbox, where
  peer-credential authentication is available;
- complete workspace all-target compilation.

Strict workspace Clippy remains blocked by the pre-existing unrelated
`moira-protocol::agent::ReplyV1` `large_enum_variant` warning. Re-running strict
Clippy over every affected package with only that known diagnostic temporarily
allowed found no new warning. No lint or source suppression was added.

## Nonclaims and remaining work

The relationship does not establish ambient Chronos scope, qualify the
task-conversation relationship, mutate a task, prove delivery, accept work,
mark Done or authorize rollout. The J1 evidence controller still needs a
qualified Pantheon composition that consumes this owner observation alongside
the exact Chronos task namespace, qualified scope/task-conversation binding and
current ReadyForReview observation. Stakeholder standing, strict evaluator and
human acceptance remain separate gates.

