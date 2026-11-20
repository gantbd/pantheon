# J1 review-link evidence verification

Date: 2026-11-20

Status: canonical design admitted; pure typed composer and read-only headless
adapter qualified across real owners, owner restart, owner unavailability and
ambiguous relationship refusal; stale/reopened live qualification and human
presentation remain open

## Bound artifacts

- canonical Rune:
  `design/runes/j1-review-link-evidence.machine.rune`
- canonical Rune SHA-256:
  `512e8be9444ef7cfed9bc8e5058bfb10c935e96513670387af6ca64f4d7d7d0d`
- Scry typed-IR SHA-256:
  `ed94cab9ff0b6c7f81636db2e34eea3d1969639effd6b315999587a074627e1e`
- pure Rust composer and read-only owner adapter:
  `cli/src/j1_review_link.rs`
- composer/adapter SHA-256:
  `88772e3292c63dc56226b8183100d950733b6e497e23b57a99eb7a2a4815ab19`
- Pantheon command route SHA-256:
  `17ea23b3c6cee6770d0e7fdd5420259b56c3113a8b1bf1d389785bed7a4e027f`
- real-owner command qualification SHA-256:
  `79d4b6fdbd6e5c6a5c408f5bb3dc832429b8bae529f878db52d0027944fb7ba0`

## Owner finding

The existing `task-conversation.v1` relationship already retains an exact
Chronos scope, task ID, incarnation, optional-at-schema attempt, Chronos observation
revision, Agora room, Agora observation revision, relationship identity,
principal and owner revision. A new owner schema is not required.

The real-owner qualification exposed two lower owner-contract defects rather
than just fixture failures. Chronos retained the terminal attempt count but
discarded it from `Succeeded` and `Failed` task projections, making an exact
post-completion attempt join impossible. Chronos now retains that content-free
attempt number in the snapshot without exposing lease authority. Moira's
task-agent journey already accepted Chronos incarnation zero, while its agent
participation and tool-continuation validators independently rejected the
same valid first incarnation. Those validators now preserve Chronos's
canonical zero while continuing to require a nonzero external task key and
attempt number.

The exact owner repairs are bound by:

- Chronos engine SHA-256:
  `fda7da588c6c86f084f7a10ffeef28c4657a70ed943c5b6739b4ad86794b87b2`;
- Chronos regression SHA-256:
  `dda303dbf525d437f727575ee6660788189a392b772168bd67ad069b5f986990`;
- Moira participation and tool-continuation SHA-256:
  `db42dd285511bb3327fec8df0d22e69ba79f2c4432a4968a51f2205f031bda25`
  and `6dd14c7dcc0ccdf34bf8a410ee5d507e33aa0b57c595b5b3f1d40007ad217c3f`;
- Moira regression SHA-256:
  `f526fb2cdb182e21b24c346a278ce61d0b4264669f16653b64efc72af1f7a22c`
  and `989d0f00aecf65a89e0c478ccd7f0eb328629bf253d631e723790ec24e7f4e95`.

The scope is relationship-owner evidence. Chronos workflow V9 publishes the
current task identity, incarnation, attempt, revision and status but does not
publish that scope. The composer therefore preserves the attribution boundary:
it does not claim that Chronos V9 issued the scope, and it does not substitute a
workspace profile, workstream display name or caller alias for the retained
relationship.

The existing Pantheon TUI relationship-creation path currently offers
`task.workstream` as the scope. That is sufficient to prepare a human-reviewed
relationship but is not independently a Chronos-issued scope fact. This
composer consumes the relationship owner's durable observation as the scope
fact and separately consumes Chronos for current workflow truth.

## Preserved proof obligations

The composer accepts only:

- one complete current task-conversation owner reply with exactly one link
  matching the owner-derived task ID, canonical incarnation, exact non-optional
  attempt and publication room; unrelated links in a shared-room observation
  are permitted, while zero or multiple matching links refuse; `link_count` is
  correctly treated as total registry cardinality rather than result
  cardinality;
- one Moira task-attempt namespace and one Moira V7 actor-participation
  relationship for the same journey;
- exact external task key, incarnation and attempt equality between the Moira
  namespace and admitted participation;
- exact task ID equality across Moira, task-conversation and Chronos;
- canonical unsigned-decimal incarnation and revision strings with no leading
  zero aliases or overflow;
- the exact actor-publication room and an unthreaded task-conversation link;
  thread-specific links refuse because the current publication evidence carries
  no thread identity;
- an actor publication whose Agora revision is not older than the retained
  room relationship;
- a validated exact-request Chronos V9 snapshot whose current revision is not
  older than the Moira attempt or task-conversation observation and whose state
  is exactly `ReadyForReview`, `Succeeded`, unblocked and on the admitted
  attempt.

The output retains the complete typed owner observations and revisions. Its
custom `Debug` representation exposes only bounded lengths and non-sensitive
status/cursor facts. It grants no publication, delivery, task mutation,
acceptance, `Done`, go/no-go or rollout authority.

## Tool and test evidence

- Smith canonical formatting passes.
- Real Scry admits the five-state, five-edge, two-terminal machine.
- Smith Rust compatibility reports `complete` with an empty unsupported set.
- `cargo test -p pantheon-cli --lib j1_review_link --locked --offline` passes
  four focused tests covering the exact success path, redaction, unrelated
  shared-room and registry entries, ambiguous exact matches, noncanonical
  incarnation, missing thread identity, room substitution, stale Chronos
  revision and non-review status.
- Strict `cargo clippy -p pantheon-cli --lib --tests --locked --offline -- -D warnings`
  passes in an isolated Cargo target directory. The first shared-target attempt
  was stopped after six compiler processes remained in uninterruptible
  filesystem I/O for more than six minutes; no diagnostic had been emitted.
- A real production-command test starts Chronos, Moira and task-conversation
  owners, creates and settles the exact first-incarnation task, observes the
  exact actor-publication relationship, invokes the shipped Pantheon binary,
  cleanly restarts all three owners and obtains the identical typed evidence.
  It then proves a stopped Chronos owner emits structured
  `chronos_unavailable` guidance and two exact relationships in one shared room
  emit structured `AmbiguousTaskConversationObservation` refusal. No partial
  observation is promoted in either failure.
- The full affected Chronos coordination test file (nine tests) and Moira
  participation/tool-continuation test files (ten tests) pass. Strict Clippy
  with warnings denied also passes for all three affected owner packages.

`machine_path_check` nominated `review-link-evidence-issued` and correctly
reported the legitimate `composition-refused` terminal as a second sink that
cannot reach the success terminal: five states, five edges, two reported
failures. The refusal branch was not removed to satisfy a single-terminal
checker.

## Remaining work

The `j1-review-link-observe` command now implements the intended read-only
route. Its only semantic selector is the exact Moira journey identity. It reads
the namespace and actor-participation relationship through Moira's public
client, queries task-conversation by the owner-derived room, selects the unique
exact task match, queries Chronos V9 by the owner-derived task ID, invokes the
pure composer, and returns a content-free structured receipt. It accepts no
caller scope, task, room, actor, attempt or revision alias. Typed composition
refusals include an orienting next action, and the Pantheon binary compiles with
the route enabled.

This is now a qualified operational headless proof leaf for the observed
states above. The next slice must:

1. qualify a real stale/reopened Chronos sequence rather than relying only on
   the pure composer's stale and non-review-state laws;
2. present the same evidence in the human review surface without implying
   acceptance or `Done`.

The first item is linked to the existing Chronos backlog owner
`chronos-task-lifecycle-transition-controls` (live revision 83, incarnation
69), not a duplicate J1 ticket. Source audit confirms why it is a real
dependency: `Engine::transition_task_status` has no public owner protocol or
client, and its nominal `ReadyForReview -> InProgress` edge is rejected by the
same function because the completed task remains `Succeeded` while
`InProgress` requires `Running` or `Uncertain`. Rejection/reopen semantics must
therefore be defined and exposed with human authority and receipts before this
proof can be run. The existing
`chronos-blocked-resolution-link-contract` owns the corresponding inspectable
resolution-link requirement.

The headless adapter itself is no longer the J1 hole. Current-status evolution
and human presentation remain `Maybe` at the next lower boundaries.

The enriched J1 authoring request has identity
`a6d4fde8ee537ae7f19371b8bf0b35f393a1900352b50d6089cd2372a05af23e`.
The real `rune-author` returned `questions_pending` and no candidate; its
remaining question names real stale/reopened owner-status evolution rather
than asking for another relationship schema, comparison law, adapter design or
already-qualified restart/unavailability/ambiguity behavior.
