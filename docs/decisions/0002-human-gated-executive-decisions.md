# ADR 0002: Human-gated executive decisions

Status: accepted

Date: 2026-11-17

## Context

Automated evidence, policy and stakeholder-SME review can improve a decision but
cannot silently assume human accountability. Final rollout changes the
authoritative/default production population. Other material decisions may also
require human judgment when a credible human-advocate lens identifies affected
people, residual risk, redress or organizational accountability.

## Decision

Final production rollout always requires presentation to and an explicit
go/no-go from a named human principal. Silence, timeout, prior sponsorship,
observer success, ensemble agreement, automated policy or an agent response is
not approval.

For other material decisions, an admitted supported and non-frivolous
human-advocate finding creates `HumanGateRequestV1`. It pauses the exact decision
until a named human records go, no-go, revise or defer under current authority.
The advocate triggers presentation but gains neither veto nor effect authority.
The work owner cannot bypass presentation through relabeling or automatic
disposition. Materially changed inputs invalidate the response.

An Executive Sponsor is the accountable human for business outcome, funding and
organizational commitment in sponsored work modes. Sponsorship does not replace
owner evidence, Aegis authorization, independent review or the fresh final
rollout decision.

## Consequences

- TUI and headless contracts expose the same gate, but automation cannot
  synthesize the human response.
- Moira retains decision presentation/response; Aegis evaluates authority; the
  semantic owner executes the resulting exact command.
- Human gates must remain accessible, attributable, revision-bound and
  inspectable after completion.

