# J1 strict review-evidence verification

Date: 2026-11-20

## Claim

Pantheon can deterministically issue one content-free evidence identity only
when already-qualified owner observations prove that the exact Daimon result
for the exact terminal Moira participation and admitted task attempt is the
exact Moira-attributed Agora message in the exact task conversation while
Chronos remains `ReadyForReview` after a successful execution.

This is a Maybe Machine boundary. A missing, stale, evicted, truncated,
ambiguous, substituted or conflicting observation is a typed refusal. It does
not become a guessed Boolean and cannot prepare a human disposition or mutate
Chronos.

## Authored machine

Source: `design/runes/j1-strict-review-evidence.machine.rune`.

The machine has five states and five edges. Its success sink is
`strict-review-evidence-issued`; its independent refusal sink is
`composition-refused`.

Scry accepted the exact source:

```text
../rune/src/ocaml/_build/default/scry.exe \
  design/runes/j1-strict-review-evidence.machine.rune
exit: 0
```

Smith canonical formatting accepted the source without rewrite:

```text
../rune/bin/smith fmt --check \
  design/runes/j1-strict-review-evidence.machine.rune
exit: 0
```

Smith's Rust compatibility check reported `status: complete`, no unsupported
guarantees, and preservation of transition structure, invalid-pair refusal,
source/IR provenance, signal directionality, compelled-edge causality,
content-free observation, target verification, preconditions, postconditions
and rung enforcement:

```text
../rune/bin/smith compatibility --target rust \
  design/runes/j1-strict-review-evidence.machine.rune
exit: 0
```

The single-success-sink path checker reports two intentional failures:

```text
terminal sinks are [strict-review-evidence-issued, composition-refused],
expected [strict-review-evidence-issued]
state composition-refused cannot reach strict-review-evidence-issued
```

That result is retained, not repaired. `composition-refused` is terminal by
design: The Gray is the Maybe Machine, and unresolved evidence cannot silently
transition into Bool.

## Rust projection

`cli/src/j1_strict_review.rs` is a pure read-only composer. It rechecks the
qualified review state and admitted task attempt, requires the exact current
Moira participation to be terminal `Completed`/`EndTurn` with no active or
pending work, and binds its admitted session, terminal turn, last cursor and
output digest exactly to Daimon's result. It then requires one complete
identified-message observation with:

- the exact owner-issued room, message ID and message sequence;
- a current-enough Agora revision and a retention frontier covering the
  selected message;
- no continuation cursor and no duplicate exact identity;
- the exact Moira-attributed actor;
- body bytes exactly equal to the bounded Daimon result; and
- an independently recomputed SHA-256 digest equal to Daimon's owner-bound
  digest.

The domain-separated canonical receipt contains no result prose. It binds the
Moira journey, current terminal participation, authority, method and grounding
identities;
task-conversation provenance; current Chronos reference and revision; Daimon
session, turn, execution, task attempt, terminal interval and output digest;
and Agora room revision, retention frontier, message identity, sequence and
author. Variable fields use explicit length prefixes and optional fields use
explicit presence tags; enum tags are exhaustively mapped.

Focused tests:

```text
cargo test --manifest-path cli/Cargo.toml --lib \
  j1_strict_review::tests --no-default-features

7 passed; 0 failed; 132 filtered out
```

They qualify stable exact evidence and refuse participation, session, turn,
cursor, terminal digest, task-key, incarnation, attempt, message-ID, author,
body, revision, retention, continuation, duplication and independently
corrupted digest substitutions. A changed owner revision produces a different
receipt. The strengthened test first exposed that the former valid fixture's
Daimon session did not equal its Moira participation session; correcting only
that fixture made the newly visible owner join explicit.

Strict focused Clippy:

```text
cargo clippy --manifest-path cli/Cargo.toml --lib \
  --no-default-features -- -D warnings
exit: 0
```

The first cold targeted test build took 8m51s while the tests took 0.00s;
the cached rerun compiled the changed CLI in 5.14s. This is evidence of broad
build-topology cost, not evidence that the proof needs a broader recurring test
gate.

## Operational projection and remaining nonclaims

`j1-strict-review-observe` now projects the read-only sequence through public
owner clients without accepting task, participation, session, turn, room,
actor, message or result aliases. Its deployment inputs are the distinct Moira
journey and participation sockets, task-conversation socket, Chronos directory,
Daimon and Agora sockets, deployment-fixed Agora participant/transport, and
journey identity. The separate Moira sockets are required because journey and
participation deliberately use distinct wire profiles.

The command observes the initial review link, derives the exact participation,
derives Daimon session and terminal turn from that participation, qualifies the
owner-bound Daimon result, folds at most eight revision-consistent, globally
ordered and identity-unique 64-message Agora identified-message pages,
reobserves the exact Moira participation, and
finally reobserves the review link and Chronos state before calling the strict
composer. Missing arguments and owner refusals produce versioned actionable
JSON whose boundary leaves the unresolved join in the Maybe Machine. Three pure
tests qualify the bounded page fold and refuse cross-revision, cross-room,
overlapping-sequence or repeated-identity folding. Existing J1 result tests,
review-link tests, the real review-link
restart gate, binary checks and strict all-target default-feature Clippy pass.

## Real multi-owner gate

The production-command integration now composes actual Unix owner boundaries
for the Moira journey and participation profiles, Daimon result observation,
Agora identified-message observation, task-conversation relationship and
Chronos workflow. The fixture creates one real terminal participation and
owner-bound result, publishes the exact result bytes through Agora's engine and
authenticated host, and invokes the production `j1-strict-review-observe`
binary. It requires `strict_review_evidence_issued` with the exact task,
participation, actor and content.

The gate then stops and reconstructs every restartable owner from durable state
or the same deterministic owner facts and requires the identical strict receipt
digest. With Chronos stopped, the same command must emit versioned structured
refusal JSON whose boundary retains the unresolved join in the Maybe Machine.
That refusal terminates the observation attempt; it is not the parent claim's
Boolean `false`.

```text
cargo test --manifest-path cli/Cargo.toml \
  --test j1_review_link_command -- --nocapture

1 passed; 0 failed; runtime 1.52s
```

Strict target Clippy also passes with warnings denied. On this host, relinking
the enlarged debug integration target took 4m29s and strict dependency checking
took 4m52s while the journey itself took 1.52s. This is a qualification-harness
cost and reuse signal, not domain latency; recurring gates should remain
focused and eventually share owner fixtures without weakening their process
boundaries.

This closes successful live strict composition, same-evidence restart,
unavailable-owner refusal and real stale/reopened owner-state refusal. The
second gate applies a genuine Chronos V10 `RevisionRequired` with feedback
evidence. While the task is reopened as `Open`, the strict command refuses; the
historical first attempt is correctly absent from the current-attempt
projection. A real worker then completes attempt two and returns the task to
`ReadyForReview`; the unchanged Moira attempt-one admission still refuses with
`AttemptMismatch` rather than silently retargeting.

Both gates pass together:

```text
2 passed; 0 failed; runtime 2.11s
```

UADD packet binding, verified reviewer response, Chronos V10 composition,
task-centric durable audit, TUI parity and rollout readiness remain open. Only
this qualified strict receipt may enter a separately governed UADD request for
the named human.
