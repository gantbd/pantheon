# Composition burn-down

This file orders cross-repository delivery. It links to owner roadmaps rather
than duplicating their internal task lists. Daimon's active work remains
read-only input until that repository settles.

## P0 — evidence and current parity

- [ ] Author and qualify the parent Maybe Machine: retain one exact uncertain
  claim, select among deterministic proof, owner observation, bounded
  experiment, stakeholder deliberation, human return, held plurality and safe
  refusal, and monadically fold typed evidence back without self-promoting to
  Bool. The existing `pantheon-gray-experiment` Rune is one child path, not the
  parent. Do not fake composition while Rune invocation/typed-return support is
  still an admitted toolchain gap.
- [ ] Restore state-local Tellus SME-plan admission without weakening the
  orthogonality check: the fold currently routes `rune:security`, but that
  grounding has no authored `(concern …)` clause, so
  `axon/sme/axon-concerns` correctly returns `role_missing_concern`. The Rune
  corpus owner must register the concern or explicitly declare the role
  ineligible; Pantheon must not invent a default facet or independence.
- [ ] Make Tellus load discovery continuable and semantically unambiguous. The
  current observation can return 64 fresh entries with `truncated: true` and no
  cursor while also reporting `complete`; expose continuation and distinguish
  selected-page freshness from complete inventory coverage.
- [ ] Project the new Daimon result-binding observation through the selected
  Oculus telemetry path: count request/accepted/refused/outcome-unknown,
  interval size and qualification latency; trace the opaque operation,
  execution and task-attempt correlation without assistant bytes, prompts,
  credentials or raw authority material. Telemetry failure must remain visible
  and must not change the owner result or qualification outcome.
- [x] Preserve authenticated Agora actor provenance in the J1 publication
  evidence path. `ActorPublicationAcceptedV1` proves the accepted message but
  drops the host-derived actor, while `PublishedV1` adds effect/source/room
  binding without restoring it. Agora revision `1ddf972` adds a separately
  typed, read-only, exact-request provenance observation through Host, durable
  gating, wire, daemon and public client; actor substitution conflicts. Moira
  revisions `13bfc6d` and `b475294` retain that actor and the owner-observed room
  ID beside the full `PublishedV1`. Neither receipt claims task identity,
  delivery, acceptance or completion.
- [x] Replace caller-supplied Aegis session identity with an owner observation
  at the Agora and Moira attribution boundaries. Aegis revisions `76d1a9c`
  and `15b27b6` expose only the principal retained for the exact current
  session. Agora revision `020e873` derives `VerifiedSubjectV1` from that owner
  fact and exposes a separate actor-plus-subject publication provenance
  observation. Moira revision `2482dcc` derives participation-admission
  principal from its exact authority session and consumes Agora's subject
  provenance. Configured identities are equality fences, not evidence;
  substitution refuses. These are the two owner facts required by the later
  join, not proof that the Agora session and Moira participation name the same
  agent.
- [ ] Close the remaining J1 namespace bindings at their owners. The trusted
  worker observes the external binary task key and exact Chronos task name but
  the durable journey previously dropped the name. Moira revision `9a5836a`
  exposes that binding from the separate execution receiver; revision `0c34a72`
  closes the actual J1 trusted-worker path with an additive V6 namespace record
  atomically retained beside the unchanged V1 attempt binding. Exact replay,
  changed-name conflict, V1/V2 upgrade, SQLite reopen and real Chronos recovery
  are qualified. Revision `e851edc` adds the bounded journey-scoped read-only
  protocol, service, canonical wire and client observation; exact SQLite reopen,
  redaction and authoritative absence are qualified. Qualify the separate
  Chronos scope relationship. Moira revision `3fd35de` closes the separate
  actor-to-participation owner edge with checkpoint V7: a private qualified
  `moira-agora` consumer joins the owner-derived Agora subject to the exact
  participation-admission principal only after matching the retained full
  participation blueprint, task attempt, room source/sequence and publication
  receipt. Exact replay, substitution conflict, V1/V2 upgrade, V7 SQLite
  reopen, public read-only service/wire/client and real Unix routing are
  qualified. Pantheon's admitted
  `j1-review-link-evidence.machine.rune` and focused pure Rust composer now
  consume that V7 relationship, the public namespace, one exact
  `task-conversation.v1` observation and an exact Chronos V9 snapshot. The
  composer preserves task-conversation as the scope issuer, refuses aliases,
  ambiguity, thread-specific links without thread evidence, stale revisions
  and non-`ReadyForReview` status, and passes focused tests plus strict isolated
  Clippy. Pantheon's read-only `j1-review-link-observe` command now obtains all
  semantic identities through those public owner clients and compiles through
  the production binary. A real production-command gate now qualifies exact
  success, all-three-owner restart, structured Chronos-unavailable refusal and
  ambiguous exact relationships in one shared room. That gate exposed and
  repaired two owner defects: Chronos now projects its already-retained
  terminal attempt number, and Moira participation/tool gates accept Chronos's
  canonical first incarnation zero while retaining nonzero task-key and
  attempt fences. Chronos revision `e0bb38c` now closes the owner-side status
  sequence with public review V10 rather than exposing a generic status setter.
  The canonical
  `chronos-review-disposition.machine.rune` now defines Accepted,
  RevisionRequired, Deferred, exact lost-reply reconciliation and exhausted
  budget refusal. The owner implementation gates dependency satisfaction on
  committed `Done`; Accepted atomically commits `Done` and releases eligible
  dependents, while RevisionRequired preserves attempt history and reopens only
  within the existing budget. Checkpoint V5 re-blocks unstarted dependents from
  V1-V4 state without inventing historical acceptance. Exact review commands
  and evidence digests survive lost replies and real Unix-service restart, and
  content-free review outcome signals are exposed. Remaining J1 work is the
  Pantheon headless behavior-facade command, real stale/reopened composition and
  human presentation parity. Also owner-bind durable task-centric review audit
  after bounded invocation retention expires; persistent `Done` must not be
  presented as indefinitely self-proving once its command-scoped receipt is
  evicted. Pantheon must not hash an unbound task name, trust deployment
  configuration or infer scope.
- [x] Project Daimon's content-free terminal agent-result binding through
  Moira's independent public agent protocol. Revision `c6c31ac` adds a read-only
  exact session/turn command, complete result DTO, canonical wire, client,
  authorization and Daimon adapter projection. It preserves execution, task
  attempt, principal/profile, operation/authority, prompt/context digests,
  terminal interval, disposition and output digest; canonical wire, adapter,
  client and downstream all-target checks pass. It grants no publication, task
  mutation, acceptance, retry or rollout authority.
- [ ] Make `rune-author` question output path-stable and nonduplicating without
  semantic loss. The revised J1 request has four unique unresolved paths, but
  `$.review.ensemble_fit` is emitted once from the explicit evidence-rich hole
  and again by generic review validation. Prefer/merge the specific authored
  question by path; never ask users to delete the richer hole or mark ensemble
  fit `yes` merely to silence duplicate diagnostics.

- [x] Define the standalone/Pantheon pattern and on-disk taxonomy.
- [x] Classify Axon and Engram as prior art and route their useful patterns to
  current owners.
- [x] Close Aegis interactive and headless Pantheon attachment qualification
  using the existing `authority.observe.v1` peer and real durable daemon.
- [x] Qualify Moira, Daimon, Hermes, Aegis and Tellus real-service observation
  together in one Pantheon invocation, not only as isolated peer attachments.
- [x] Keep the root repository inventory synchronized with all presentation
  peers and current source state.
- [x] Move reusable Hermes, Agora, and Aegis Unix transport clients into their
  owner repositories; reduce TUI packages to compatibility re-exports and make
  Pantheon consume the owner clients directly. Defer Moira and Tellus until
  their active owner boundaries settle.
- [x] Keep Hermes hosting out of the mixed engine graph: Pantheon now owns the
  owner-supplied standalone daemon process and attaches the same public client,
  preserving Hermes authentication, authorization, storage and shutdown.
- [x] Resolve the Chronos/Agora Keystone source collision without forcing an
  unselected topology: production Pantheon keeps Chronos native and supervises
  Agora's exact standalone process, so the source graphs remain separate. The
  patched all-three in-process qualifier stays test-only. Any future in-process
  product must first select one immutable remotely retrievable Keystone source;
  sibling paths and duplicate vendored identities remain forbidden.

## P1 — production composition

- [x] Complete Agora's production durable owner factory without putting
  credential policy in Pantheon. Agora now owns the qualified single-principal
  private-endpoint authenticator, non-regressing Unix-epoch clock, and an
  adopted `agora-private-local` bounded create-time participant/room bootstrap.
  Aegis now supplies a service-free `aegis-local-client`; Agora connects through
  it, binds the bootstrap principal before socket I/O, and invokes a
  deployment-owned grant selector only on the exact live Aegis session. Agora's
  concrete provisioner now issues one exact conversation or
  scheduled-composition scope using deployment-supplied authority epoch,
  lifetime, resource, subject, and administrator credential mapping. The owner
  factory now orders durable create, clean open, and exact endpoint
  residue recovery, and the shared owner loop returns an exact cleanup receipt
  after explicit shutdown and drain. The bounded standalone configuration and
  POSIX signal entrypoint reuse that owner. The real Aegis/Agora standalone
  journey now publishes/observes through the public scheduled client, drains,
  and reopens retained state. Pantheon supervises the exact same binary from
  an opaque config, uses the routed public Protocol V2 client in headless and
  interactive modes, drains Agora before Aegis, and qualifies durable reopen.
  A test-only cleanup-receipt proxy proves a non-clean Agora result refuses
  aggregate success without suppressing the later Aegis drain, followed by an
  exact durable reopen. A separate shutdown-edge child kill proves explicit
  residue preservation, safe replacement readiness, exact incarnation rotation,
  and retained-room recovery. A bounded missing-receipt cut proves
  `outcome_unknown`, continued Aegis drain, and the same exact recovery.
- [x] Define a typed Tellus presentation client over Dendrite's bounded
  observation/provenance behavior, separate from MCP.
- [x] Add `tellus` and its standalone interactive/headless application,
  then attach the same component to Pantheon.
- [x] Keep Tellus observation-only for this program. Selected surface/read
  DTOs are deliberately unselected until Dendrite owns exact content
  disclosure, provenance, freshness and response bounds; the current typed
  observation and standalone/Pantheon attach contract is complete without
  importing its MCP authority.
- [x] Add deployment-owned typed lifecycle factory seams for Moira, Daimon,
  Hermes, Aegis, and Tellus while preserving separate state, authority,
  recovery and shutdown receipts.
- [x] Promote Moira through its owner-supplied sovereign `moira-saga-local`
  profile after explicit storage, Aegis authentication, recovery and shutdown
  composition passed the standalone and Pantheon real-process lifecycle
  qualification. Pantheon supervises the exact owner binary from an opaque
  profile and cannot upgrade its reconciliation authority. Tellus remains
  attach-only by design: Dendrite retains its IDE/agent bridge lifecycle.
- [x] Promote Daimon's production local-agent application to an explicit
  supervised profile factory after the owner supplied an opaque configuration,
  exact public agent socket and one-byte drain contract. Pantheon does not
  acquire provider, credential, authority, attempt, storage or recovery policy.
- [x] Hermes: qualify the owner-supplied selected daemon under Pantheon
  lifecycle for explicit creation, recovery, public-client observation and clean
  signal drain without linking engine or substrate internals. The selected
  operator gate also qualifies exact-confirmed acknowledgement and compaction,
  followed by durable reopen, through the same public clients.
- [x] Aegis: qualify the owner-supplied daemon under Pantheon lifecycle with
  explicit authority/principal/session/role facts, create/open recovery, the
  ordinary public client and clean signal drain. Pantheon retains no grant,
  clock, authentication or authority policy.

## P2 — operational depth

- [ ] Repair Moira's existing strict-Clippy baseline without suppressing it:
  `moira-protocol::agent::ReplyV1` currently triggers
  `clippy::large_enum_variant` because its accepted outcome is substantially
  larger than refusal. Decide and qualify the compatible indirection strategy;
  the V7 actor-participation slice introduced no source or lint suppression and
  passes strict Clippy when this one pre-existing diagnostic is isolated.

- [x] Moira: carry bounded selected-saga effect evidence and operator receipts
  through the same read-only component in standalone and Pantheon interactive
  paths.
- [x] Moira: keep reconciliation mutation guarded until exact confirmation,
  current authority, revision, durable receipt, and outcome-unknown recovery
  are public and qualified end to end.
- [x] Hermes: carry a read-only cursor observation plus exact opaque stream and
  consumer selectors through standalone and Pantheon without acquiring the
  acknowledgement-capable consumer facade.
- [x] Hermes: add exact-confirmed event disclosure through a separate read-only
  facade, bounded to 16 events and the negotiated frame ceiling, with explicit
  retention/publication gaps and no global stream or consumer catalogue.
- [x] Hermes: reuse the peer-owned same-connection consumer guard for one
  exact-confirmed acknowledgement in standalone and Pantheon, fencing stale
  cursors and never retrying an uncertain mutation.
- [x] Hermes: reuse the peer-owned exact-stream-state guard for one compaction
  in standalone and Pantheon, keeping operator authority out of the reusable
  component and selecting any hosted operator authority explicitly at the
  Hermes process boundary.
- [x] Aegis: reuse the peer-owned exact-confirmation/current-epoch guard for
  one-attempt epoch revocation in standalone and Pantheon, preserving
  `outcome_unknown` without retry.
- [x] Aegis: reuse the owner-supplied bounded administration-session protocol
  for named issue/attenuate/authorize/list/close in standalone and Pantheon;
  grant handles remain session-local and private to the shared manager.
- [x] Aegis: qualify explicit disconnect/reconnect abandonment and deliberate
  rebuild under a new operation name in both standalone and Pantheon shells.
  Durable grant replay and lost-reply reconciliation remain nonclaims.
- [x] Aegis: preserve bounded redacted grant/decision pagination through the
  same read-only component in standalone and Pantheon interactive paths.
- [x] Chronos: reuse its public Hub and owner service for task creation,
  receipted cancellation, scheduling enablement, global/task pause, capacity,
  and not-before controls through Pantheon; prove clean drain and exact
  independent durable reopen without importing mutation semantics.
- [x] Daimon: reuse exact configured agent-session/provider evidence through
  standalone and Pantheon without exposing a session catalogue or mutation
  client.
- [x] Daimon: add guarded prompt, advance, permission, cancellation, close and
  reattach only with
  current authority, exact confirmation, durable reconciliation and no-retry
  qualification.
- [x] Daimon: close the integrated hosted lifecycle gate across the production
  owner process, ordinary public client, clean drain and durable reopen.
- [x] Chronos: qualify the selected task and scheduling mutation profile through
  the same public Hub and owner service in standalone and Pantheon paths,
  preserving peer-owned receipts, clean drain and exact durable reopen.
- [x] Vellum: qualify the implemented `workspace.edit.v1` and
  `workspace.save.v1` path in standalone and Pantheon. Real-owner gates
  preserve revision witnesses, dirty/clean/indeterminate projection, save
  authority refusal, exact persisted bytes and exact replay after a deliberately
  discarded accepted reply. Newer durable operations remain a separate adapter
  selection item.
- [x] Agora: qualify selected `rooms.create.v2` through one owner-client guard
  in standalone and Pantheon. The command binds the exact room facts and
  current revision, refuses an inexact confirmation before connection, returns
  the durable owner receipt, and resolves a discarded reply only by replaying
  the identical retained request identity. Broader Agora mutations remain
  unclaimed rather than becoming generic Pantheon commands.

## P3 — cross-engine completion

- [x] Record Hermes append at its actual owner boundaries rather than inventing
  a Pantheon command: Moira qualifies its workflow-event schema and
  `agora-hermes-adapter` qualifies minimized-wake append/replay. Generic append
  is not a presentation-parity requirement.

- [x] Close Agora–Chronos's adapter-owned Aegis mapping through five explicit
  session-local grant selections: Chronos Submit/Observe/Modify and Agora
  Execute/Observe over the opaque publication resource. The public-client
  mapping is qualified against the real Aegis daemon with fresh trusted time,
  fail-closed replies, live epoch revocation, and clean drain.
- [x] Close Agora–Chronos atomic admission/eligibility and lost-reply recovery:
  Chronos V7 publishes graph plus exact-incarnation eligibility in one retained
  transition, command-scoped lookup cannot admit missing work, and V4 returns
  the original allocation. The same public client is qualified with real Aegis
  and Chronos daemons through a deliberately discarded reply.
- [x] Add retained Chronos V8 cancellation with exact-reference guards and a
  non-admitting scoped lookup; the real dual-daemon gate discards and recovers
  both admission and cancellation replies without a second mutation.
- [x] Preserve the post-offer cut: even known Chronos cancellation remains
  reconciliation-required and is never treated as proof Agora did not publish.
- [x] Add Agora's actor-bound, read-only retained-observation owner contract
  and adapter reconciliation: accepted evidence wins; authoritative absence
  closes only with independently terminal Chronos cancellation.
- [x] Deploy Agora publish/observation through private-local composition
  handshake profile 4 and the public `ScheduledLocalClientV1`; ordinary
  participant Wire V2 remains unchanged. The Agora–Chronos adapter wraps the
  same transport-neutral client for standalone and Pantheon use, preserves
  pre-send versus outcome-unknown evidence, and keeps its Dispatch/InspectAgora
  decisions separate from Agora's owner-side Execute/Observe enforcement.
  The isolated host qualification also returns this exact client from
  `NativeHost` and proves publish/observe under Pantheon lifecycle with
  explicitly test-only authority. Supplying the production owner factory
  remains part of the P1 Agora provenance/grant/time/durability item. The
  adapter's local release profile is separately closed over pinned clean Aegis,
  Agora, Chronos and Keystone revisions, optimized tests, 10,000 checkpoint
  mutations, 10,000 recovery cycles, and the real lost-reply/revocation gate.
- [x] Close the Agora–Hermes concrete endpoint and deployed local production
  profile. Agora now exposes its existing content-minimized projection through
  the sovereign composition client and durable daemon. Hermes supplies an
  exact-stream delivery profile with append, cursor observation and
  acknowledgement but no payload read, compaction, other-stream or service
  authority. The separately classified `agora-hermes-local` application verifies
  private same-owner sockets, obtains current Aegis epoch, issues only the
  create/recovery actions it needs, uses non-regressing Unix-epoch time, owns
  separate durable journal and bounded receipt-spool directories, and drives
  only the public Agora and Hermes clients. The real gate runs explicit
  separate Agora-participant and delivery-subscription Aegis realms plus Agora
  and Hermes owner processes, proves durable endpoint receipt before Hermes
  acknowledgement, exact application reopen and clean owner drain. The selected
  endpoint has no network, URL, token, secret or payload field; remote-provider
  profiles and cross-realm revocation remain explicit nonclaims.
- [x] Complete Moira's process-separated Agora/Vellum workflow and every-boundary
  lost-reply matrix. The Agora response segment is now closed independently:
  Agora retains a caller-stable actor operation, its durable daemon commits one
  addressed response in a separate process, Moira discards the successful reply,
  and a fresh authenticated public client observes the exact result without
  republishing. That proof now uses the exact standalone Aegis and Agora owners,
  current bounded grants, and ordered clean shutdown. The Vellum segment is now
  independently closed as well: its
  exact standalone authenticated daemon commits one revision-bound durable edit,
  loses the reply, and a fresh public client performs historical lookup without
  execution fallback before public control-socket drain. The combined gate now
  admits both effects into one SQLite-backed Moira saga against the exact
  standalone Aegis, Agora, and Vellum owners, discards both accepted replies in
  provider order, reopens Moira between fences, reconciles Agora only through
  actor observation and Vellum only through historical lookup, and proves
  completed replay makes no peer call. The wider ingress/execution/event fault
  chain remains Moira-owned roadmap work rather than part of this peer-effect
  composition cell.
- [x] Complete physical authority/transport/revocation composition for Chronos
  and Vellum federation. Each federation now owns a peer-specific TLS 1.3
  adapter with mandatory mutual certificates, distinct ALPN and exporter
  labels, canonical hello exchange on the authenticated channel, and an
  exporter-bound session. Their transport-neutral connection owners revalidate
  current trust immediately before every selected transport use, make
  revocation terminal, hide revoked sessions, and send close notification
  exactly once before refusing the operation; Vellum applies the same rule to
  disclosure downgrade. Full repository gates qualify peer EOF after live
  revocation. Listener supervision, reconnect/backoff, certificate lifecycle,
  and the broader graph/result or replication/edit data planes remain
  federation/deployment work rather than claims of this cell. Agora
  federation's immutable release qualification is complete: its exact source
  candidate and Aegis, Agora,
  remote-transport, and Keystone inputs are reconstructed from Git archives,
  the full physical mutual-TLS/current-policy/durable gate passes, and an
  evidence-only follow-up records that candidate without changing it.
- [x] Install Vellum's selected newer durable operation only through its owner
  authority and platform adapter. `workspace.durable-save.execute.v1` and
  `workspace.durable-save.lookup.v1` share one Vellum-TUI guard over the public
  Vellum client in standalone and Pantheon. The caller persists the operation
  reference, while the authenticated daemon resolves the exact current
  document, enforces its Persist ceiling, performs filesystem work, and owns
  receipt truth. A real-daemon gate executes in standalone and recovers the
  exact receipt through Pantheon. Save As submission, receipt retirement,
  arbitrary destinations and other durable intents remain unselected and are
  not implied by this cell.

## P4 — distribution and additional presentation

- [x] Capture immutable peer and engine revisions and rerun the full standalone,
  attach and host matrix.
- [x] Close source/legal review, upgrade/migration, supported-platform and
  artifact gates for the selected local deployment. The immutable source
  qualifier builds an unsigned `aarch64-apple-darwin` tarball on the recorded
  Darwin 25.6.0 arm64 host, emits a SHA-256 file manifest and exact reviewed
  transitive license inventory, and qualifies initial install plus atomic
  same-version replacement without modifying the external version-1 profile.
  Pantheon owns no peer durable state and refuses unknown profile schemas;
  remote publication, wider platforms, signing and notarization are outside
  the selected local artifact claim.
- [x] Qualify the Pantheon GUI local dependency closure across clean committed
  Chronos/Vellum engines and GUI peers.
- [x] Defer Pantheon GUI distribution, signing and notarization by explicit
  product decision. Immutable prepared-artifact
  qualification is complete at Pantheon GUI commit
  `fdf4287dbe694f7230085c6017727bc512b63a9e`: the Apple-silicon `.app`
  reconstructs exact Chronos/Vellum engine and GUI inputs, carries a SHA-256
  manifest plus 204-package/118-text legal closure, and exposes a fail-closed
  hardened signing, notarization, stapling and Gatekeeper path. Signed release
  is deferred because the qualification host has no valid Developer ID
  Application identity or notarization profile. No signed-distribution claim
  is made, and the prepared gate remains ready for a future program.
- [x] Defer remote deployment and wider GUI coverage. They are outside the
  selected local standalone-and-Pantheon delivery and may be reopened only by
  a later product decision with concrete deployment, credential, signing and
  support profiles.

## P5 — single-machine management

- [x] Classify exact route capabilities by plane and keep guarded controls
  discoverable without admitting them to aggregate observation routes.
- [x] Expose bounded local runtime status, live observation, certainty/fencing,
  revisions where available, and exact reverse-drain receipts over stdio.
- [x] Correlate observations, drains, and restarts with a bounded local operation
  identifier that is never reused as peer identity, authority, or idempotency.
- [x] Preflight a whole replacement profile before draining, fence on any
  non-clean shutdown, and retain the stdio supervisor channel across exact
  Pantheon process replacement.
- [x] Defer service-manager, authenticated IPC, fleet, remote, and GUI
  management profiles until a concrete deployment and credential model exists.

## Exit evidence for every claimed cell

For each claimed cell record the exact source state, public client, capability
identifier, topology, authority inputs, real-service scenario, refusal and
outcome-unknown behavior, shutdown result and explicit nonclaims. A passing
read cell cannot close a write cell, and an attached cell cannot close a hosted
cell.
