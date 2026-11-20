# J1 UADD acceptance composition request

Date: 2026-11-20

Status: owner request; not an admitted Rune machine, Moira or Chronos schema
change, human response, task disposition, acceptance receipt, `Done` authority,
go/no-go decision or rollout authorization

## Driving intent

Close one J1 task only when the named user or authorized delegate has
adversarially challenged and then separately accepted the exact delivered
result identified by Pantheon's qualified strict J1 receipt. Preserve Rune's
canonical User Acceptance-Driven Development machine rather than reducing it
to a generic approval dialog or creating a parallel acceptance method.

Pantheon already qualifies the exact task attempt, terminal Daimon result,
Moira participation and actor publication, Agora message,
task-conversation relationship and current Chronos `ReadyForReview` state.
Moira already owns revision-bound human-gate requests and Aegis-authorized
responses with owner-resolvable decision-packet and rationale references.
Chronos already owns atomic review V10 with lost-reply lookup. The missing
boundary is a typed composition proving that the same named human challenged
the same delivered version, all findings were disposed, and a later response
is the UADD acceptance decision eligible for one exact Chronos disposition.

## Governing prior art

- Rune's canonical
  `rune/fsm/user-acceptance-driven-development.machine.rune` requires the named
  user or authorized delegate to act as the adversary and then make a separate
  explicit acceptance decision over the same verified version. Green tests,
  an unanswered request and a successful adversarial challenge are not
  acceptance.
- Moira's `HumanGateRequestV1` retains reviewer, finding, decision,
  material-revision and policy digests plus one owner-resolvable decision
  packet. `HumanGateResponseV1` adds an Aegis-injected authority subject and an
  owner-resolvable rationale. Those generic fields do not by themselves prove
  UADD challenge completion or authorize a Chronos mapping.
- Pantheon's strict J1 receipt is read-only evidence identity. It grants no
  human disposition, task mutation, `Done`, go/no-go or rollout authority.
- Chronos review V10 is the sole task-status effect boundary. A direct engine
  method, generic status setter or caller assertion is not a substitute.

Live Tellus/Dendrite grounding was not claimed in this authoring pass. The
existing loopback service requires a selected credential route that is not
safely available in this execution context. The request therefore preserves a
mandatory Tellus/ensemble review before admission rather than inventing SME
output or bypassing authentication. The exact future ambient-expertise request
is retained in `design/requests/j1-uadd-acceptance.sme-plan.json`; it routes
architecture, network, identity, data, adversarial testing, user experience,
humanism and live-verification concerns. Those Dendrite roles are advisory
expertise, not substitutes for the equitable, attributable owner/stakeholder
ensemble that Moira must admit separately.

## Rosetta

- **Intent:** bind one exact qualified J1 delivery to a named-user adversarial
  challenge, complete finding disposition and a later explicit acceptance
  decision, then prepare at most one owner-specific Chronos review operation.
- **Values:** user agency, same-version identity, separation of duties,
  adversarial independence, finding accountability, owner preservation,
  lost-reply safety, Maybe preservation and honest closure.
- **Outcomes:**
  1. one owner-issued UADD request binds the named acceptance principal, strict
     J1 receipt, exact challenged bytes/material reference, task attempt,
     criteria, threat inventory, policy and decision-packet identity;
  2. one separately attributable challenge receipt records the actual user's
     bounded STRIDE attempts, routes, protected effects, conclusions and exact
     delivered-version identity without implying acceptance;
  3. finding triage proves every current-slice blocker resolved against a new
     strict receipt and every adjacent finding durably linked to an owner,
     evidence and priority;
  4. a later Aegis-authorized response from the named acceptance principal
     binds the same request, strict receipt, challenged version and current
     rationale;
  5. only the closed typed composition may prepare one exact Chronos V10
     operation: `Approved` to `Accepted`, `ChangesRequested` to
     `RevisionRequired`; `Deferred` remains in the Maybe Machine and `Rejected`
     remains a distinct non-mutating UADD outcome until an explicit owner
     contract defines another effect; and
  6. Chronos acceptance, lookup or typed refusal is retained separately from
     the human and UADD receipts and never implies rollout.
- **Who:** the named human sponsor/user or authorized delegate as acceptance
  authority and adversary; Moira as human-gate and deliberation owner; Aegis as
  command authority owner; Pantheon as effect-free composer/presenter; Chronos
  as task-status owner; Vellum as selected packet/rationale material owner; and
  independent security, user-advocacy and audit reviewers.
- **What:** one bounded UADD-to-J1 evidence composition plus a closed
  Chronos-review adapter; not a new generic approval system.
- **When:** after strict J1 evidence qualifies and before any J1 acceptance,
  `Done`, dependency release or closure claim.
- **Where:** Moira UADD/human-gate evidence, Pantheon's read-only composition
  boundary, selected Vellum material routes and Chronos review V10.
- **Why:** a generic `Approved`, green test suite, agent recommendation or
  challenge result cannot prove that the actual user accepted the exact result
  after trying to break its introduced controls.
- **How:** reuse the canonical UADD state structure; bind exact typed owner
  receipts; reobserve the strict J1 evidence immediately before challenge,
  acceptance preparation and Chronos mutation; admit only a verified human
  authority subject matching the request; preserve every non-success outcome;
  execute or look up one stable Chronos invocation without blind retry.
- **Extent:** one J1 attempt and one acceptance cycle. No arbitrary task
  mutation, generic approval inference, agent-authored human response,
  automatic timeout, finding suppression, direct filesystem/shell fallback,
  go/no-go or rollout authority.
- **Example:** the accountable human tries the exact task result against the
  disclosed spoofing and tampering routes, records no bypass plus one adjacent
  usability finding with an owner, then separately approves strict receipt
  `E`; Pantheon prepares one V10 `Accepted` command bound to `E` and the UADD
  receipts, and Chronos alone commits `Done`.

## Required owner decisions

1. Which owner issues the UADD request and challenge receipt without importing
   Rune's entire activity model into Pantheon or treating Pantheon as a truth
   owner?
2. How does the request bind the strict receipt and human-readable challenged
   bytes while preserving Moira's opaque material-owner references and content
   minimization?
3. Which typed fields distinguish user-adversary attempt, conclusive challenge,
   challenge unavailable and challenge inconclusive?
4. How are all six STRIDE dispositions, attempted routes, protected effects,
   countermeasure evidence and independent corroboration bounded without
   retaining secrets or unsafe payloads?
5. How does finding triage bind resolved current-slice findings to a new strict
   receipt while retaining adjacent Chronos work links without making backlog
   admission acceptance evidence?
6. Must the adversary and acceptor be the same principal, or may an authorized
   delegate act under a separately explicit policy? Either answer must be
   owner-issued and visible to the human.
7. How is a Moira generic human-gate response typed as a UADD response so that
   unrelated `Approved` evidence can never enter the adapter?
8. Which exact response-to-Chronos mapping is owned, and what is the durable
   disposition of `Rejected`?
9. Which stable invocation identity and lookup contract survive a lost Chronos
   reply without reusing an old human response after strict evidence changes?
10. Which event, log, metric and trace fields expose request, challenge,
    findings, human response and task-effect latency/refusal without recording
    result bytes, attack payloads, rationale text, credentials or authority
    secrets?

## Mandatory refusal evidence

Qualification must refuse at least:

- missing, stale, changed, ambiguous or non-current strict J1 evidence;
- a request that does not bind the exact task attempt, strict receipt,
  challenged material, criteria, threats, policy and named acceptance principal;
- challenge evidence authored by an agent or presenter as though it were the
  human's attempt;
- absent, inconclusive, unsafe, wrong-version or wrong-principal challenge;
- any applicable STRIDE category without a disposition, or a credible threat
  without countermeasure/test/challenge evidence;
- unresolved current-slice findings, missing adjacent-finding owner/evidence/
  priority, or a changed implementation without a new strict receipt and new
  challenge;
- silence, timeout, green tests, ensemble agreement, successful challenge or a
  generic Moira `Approved` used as acceptance;
- response/request, reviewer/authority, packet/rationale, material revision,
  receipt, task-attempt or policy mismatch;
- `Deferred` or `Rejected` mapped to `Done`, and any unsupported disposition
  mapping;
- stale Chronos revision, changed current attempt, non-`ReadyForReview` state,
  exhausted/unknown invocation identity, conflicting lookup or blind retry;
  and
- any claim that human acceptance authorizes production rollout.

## Minimum qualification matrix

1. Scry admits a J1 composition machine that preserves the canonical UADD
   challenge, finding-triage and separate acceptance ordering.
2. Smith reports complete target compatibility or refuses emission; no hand
   projection may silently omit a machine obligation.
3. Pure composition tests cover every identity and disposition substitution,
   challenge/finding gap and same-version fence.
4. A real owner gate resolves the decision packet, records one attributable
   challenge and finding disposition, accepts through Moira/Aegis, reobserves
   the same strict receipt, and commits Chronos V10 `Accepted` exactly once.
5. Real negative gates cover challenge unavailable/inconclusive, changed strict
   receipt, changes requested, deferred, rejected, stale/reopened Chronos,
   unavailable owner and lost-reply lookup.
6. Headless and TUI surfaces use the same behavior facade; the human can inspect
   challenged version, threat/finding state, authority, pending action and all
   nonclaims before review/arm/confirm.
7. Oculus-visible RED/USE and golden-signal telemetry distinguishes UADD
   request, challenge, triage, response and Chronos effect outcomes without
   sensitive content.

## Nonclaims and handoff

This request does not claim that current Moira `Approved` is UADD acceptance,
that the human sponsor has already challenged or accepted this slice, or that
J1 is `Done`. It does not choose a schema owner on that owner's behalf. The
designing ensemble must first answer “do we have the right ensemble?”, include
the actual human advocate/acceptance principal, security/threat, UX, Moira,
Chronos, Aegis, Vellum, observability and audit stakeholders, and obtain a live
Tellus-grounded review through an authorized route. Only an admitted Rune and
qualified owner implementation may advance this request.
