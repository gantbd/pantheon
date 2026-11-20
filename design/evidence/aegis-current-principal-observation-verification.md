# Aegis current-principal observation verification

Date: 2026-11-20

Status: owner observation qualified through Aegis, consumed by Agora and
Moira; the cross-owner Agora-actor-to-Moira-participation relationship remains
unproved

## Why this descent was required

Agora and Moira each paired an Aegis authorization with a separately supplied
identity. A valid decision beside a caller-supplied subject or principal did
not prove that the live Aegis session belonged to that identity. Treating the
pair as a binding would have collapsed a `Maybe` into a Boolean claim at the
composition boundary.

The bounded Rune
`design/runes/aegis-current-principal-observation.machine.rune` instead asks the
session owner for one fact: the principal retained for the exact current
session. The request has no claimed principal. Unknown or closed sessions
retain the existing typed refusal.

## Design-tool qualification

- canonical Rune SHA-256:
  `14ce3c5d1a1be4d69372cb640180df68fd93106789f23311b2ec89a24099493a`;
- Scry typed-IR SHA-256:
  `11ca3ed7cf33256611bee05f95048f408089f686e6015fe649d979d7fe2c4d89`;
- qualified Rune checkout:
  `22313327fcff68a02ddcfa18b8464dae3d3c03db`;
- Scry binary SHA-256:
  `cac27f87a01181d0b0075d752cc95cbecb8597b5091dfc0323a6345151df46b4`;
- Smith binary SHA-256:
  `3288271a2dcae7bd833ddfea15a16562c5fb4335b16e0373e34c45bfa998504c`.

Real Scry admitted the four-state, three-edge machine. `smith fmt --check`
accepted the retained source. `smith compatibility --target rust` returned
`complete`, including transition preconditions, destination postconditions,
rung enforcement and executable source/IR provenance, with an empty
`unsupported` set. This is read-only design compatibility evidence, not a
claim that Smith emitted or qualified the owner implementations.

## Owner implementations

### Aegis

Revisions `76d1a9c` and `15b27b6` add a zero-payload `CurrentPrincipal`
command to the service and both public client facades. The service projects the
principal only from its exact resolved session entry. Distinct live sessions
return distinct principals; a closed session returns `UnknownSession`.
Canonical wire round trip, truncation refusal, client facades and the complete
all-target workspace check pass.

Source evidence:

- protocol:
  `sha256:c7b7a126907ab925003bea07c4b25469faf7ff35517bc2c535146e8c23bf9b55`;
- canonical wire:
  `sha256:05da5fc57bbb48ef5f430e862068868f81d58cb83ecfd7978c9385d71d89c531`;
- service:
  `sha256:feb920704547c7473bed2e8404e476c3ffb8528e097afaf5c61e5ea70a7bd59c`.

### Agora

Revision `020e873` makes the exact-session Aegis owner response the source of
`VerifiedSubjectV1`; the configured subject is only an equality fence. Every
authentication, revalidation, scheduled and actor action checks that owner
subject. Substitution refuses before authorization. A separate additive
actor-publication subject-provenance observation retains the accepted
publication, Host-derived actor and verifier-derived subject without changing
the earlier actor-only contract.

All eight verifier tests, both scheduled-wire tests, the focused Host
provenance test, the real durable private-Unix composition test and the
complete all-target workspace check pass.

Source evidence:

- actor publication port:
  `sha256:43d1d670b9c2671f355434bf3717998013802a176c79fe2d64a4e13020f8f565`;
- scheduled wire:
  `sha256:67ab5cd86fa48090bdc86834120bd1e8e2e84ac588cfc81561263131e95fbe5c`;
- Host V2:
  `sha256:e0a36f153a92239ad57099dd676e8ed895c27a5cba09f44511a60eb32ef471dd`;
- Aegis verifier:
  `sha256:b660d6b52f63922b0fc38d3ded01119146d8e9332818b16990f7ec2116d35bab`.

### Moira

Revision `2482dcc` makes the exact-session Aegis owner response the source of
the participation-admission principal; the configured principal is only an
equality fence. It also projects Agora's separate subject-provenance receipt
without changing the actor-only method. The real Aegis public-client
participation test proves the owner-derived principal and refuses a configured
principal substitution. Both Agora-adapter provenance tests and the complete
all-target workspace check pass.

Source evidence:

- Aegis adapter:
  `sha256:7de88c0ced142fb219a021e22472148a5b6f61493872fa9bd84ee164e250d573`;
- Agora actor adapter:
  `sha256:b5e8edf9f0c4197a71f44eea980b79b8cddd72bd4be12e468b33c94bf395a36d`.

## Exact remaining unknown

These revisions establish two owner facts: the subject of the exact Agora
publication session and the principal of the exact Moira participation
admission session. They do not yet establish that those facts name the same
agent participation.

The remaining owner work is a versioned, replay-stable Moira relationship that
joins the equal owner-derived subject/principal to the exact participation,
Daimon session/profile/member/task attempt and exact Agora
room/publication/effect/message. It then needs a public read-only observation
that survives close/reopen and conflicts on every substituted identity or
receipt. No current artifact claims credential equivalence, agent identity,
task-conversation scope, human acceptance, completion or rollout authority.

