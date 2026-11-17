# ADR 0003: Reflexive rollout ownership

Status: proposed; blocks J30 implementation

Date: 2026-11-17

## Context

Pantheon cannot safely develop and replace itself if candidate code also builds,
grades, promotes and rescues itself. Current local restart mechanisms are useful
substrates but do not establish independent rollout or recovery ownership.

## Proposed decision

- GitHub Actions initially builds digest-addressed native candidates with
  provenance, subject to an observed eligibility gate and replaceable contract.
- Moira retains rollout decisions from a named human principal under Aegis.
- `pantheon-steward` verifies and executes accepted rollout decisions.
- `pantheon-observer` combines owner-native probes with external telemetry.
- A host-supervisor Recovery Policy Ledger and `pantheon-rescue` apply protected
  last-known-good policy outside the normal rollout path.

Trusted-development separates ordinary failures by convention. Security,
authority, telemetry and recovery-root changes require an OS-enforced isolated
profile.

## Decision required before acceptance

Qualify exact owners, public commands, receipts, failure independence,
maintenance/update path and macOS host integration. Define an outer maintenance
profile for Moira, Aegis, `pantheon-control`, steward, observer, recovery
machinery and their schemas/trust roots.

