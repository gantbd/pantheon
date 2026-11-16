# Composition burn-down

This file orders cross-repository delivery. It links to owner roadmaps rather
than duplicating their internal task lists. Daimon's active work remains
read-only input until that repository settles.

## P0 — evidence and current parity

- [x] Define the standalone/Workbench pattern and on-disk taxonomy.
- [x] Classify Axon and Engram as prior art and route their useful patterns to
  current owners.
- [x] Close Aegis interactive and headless Workbench attachment qualification
  using the existing `authority.observe.v1` peer and real durable daemon.
- [x] Qualify Moira, Daimon, Hermes, Aegis and Grounding real-service observation
  together in one Workbench invocation, not only as isolated peer attachments.
- [x] Keep the root repository inventory synchronized with all presentation
  peers and current source state.
- [x] Move reusable Hermes, Agora, and Aegis Unix transport clients into their
  owner repositories; reduce TUI packages to compatibility re-exports and make
  Workbench consume the owner clients directly. Defer Moira and Grounding until
  their active owner boundaries settle.
- [x] Keep Hermes hosting out of the mixed engine graph: Workbench now owns the
  owner-supplied standalone daemon process and attaches the same public client,
  preserving Hermes authentication, authorization, storage and shutdown.
- [x] Resolve the Chronos/Agora Keystone source collision without forcing an
  unselected topology: production Workbench keeps Chronos native and supervises
  Agora's exact standalone process, so the source graphs remain separate. The
  patched all-three in-process qualifier stays test-only. Any future in-process
  product must first select one immutable remotely retrievable Keystone source;
  sibling paths and duplicate vendored identities remain forbidden.

## P1 — production composition

- [x] Complete Agora's production durable owner factory without putting
  credential policy in Workbench. Agora now owns the qualified single-principal
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
  and reopens retained state. Workbench supervises the exact same binary from
  an opaque config, uses the routed public Protocol V2 client in headless and
  interactive modes, drains Agora before Aegis, and qualifies durable reopen.
  A test-only cleanup-receipt proxy proves a non-clean Agora result refuses
  aggregate success without suppressing the later Aegis drain, followed by an
  exact durable reopen. A separate shutdown-edge child kill proves explicit
  residue preservation, safe replacement readiness, exact incarnation rotation,
  and retained-room recovery. A bounded missing-receipt cut proves
  `outcome_unknown`, continued Aegis drain, and the same exact recovery.
- [x] Define a typed Grounding presentation client over Dendrite's bounded
  observation/provenance behavior, separate from MCP.
- [x] Add `grounding-tui` and its standalone interactive/headless application,
  then attach the same component to Workbench.
- [ ] Extend Grounding with selected surface/read DTOs only after exact content
  disclosure, provenance, freshness and response bounds are fixed.
- [x] Add deployment-owned typed lifecycle factory seams for Moira, Daimon,
  Hermes, Aegis, and Grounding while preserving separate state, authority,
  recovery and shutdown receipts.
- [x] Promote Moira through its owner-supplied sovereign `moira-saga-local`
  profile after explicit storage, Aegis authentication, recovery and shutdown
  composition passed the standalone and Workbench real-process lifecycle
  qualification. Workbench supervises the exact owner binary from an opaque
  profile and cannot upgrade its reconciliation authority. Grounding remains
  attach-only by design: Dendrite retains its IDE/agent bridge lifecycle.
- [x] Promote Daimon's production local-agent application to an explicit
  supervised profile factory after the owner supplied an opaque configuration,
  exact public agent socket and one-byte drain contract. Workbench does not
  acquire provider, credential, authority, attempt, storage or recovery policy.
- [x] Hermes: qualify the owner-supplied selected daemon under Workbench
  lifecycle for explicit creation, recovery, public-client observation and clean
  signal drain without linking engine or substrate internals. The selected
  operator gate also qualifies exact-confirmed acknowledgement and compaction,
  followed by durable reopen, through the same public clients.
- [x] Aegis: qualify the owner-supplied daemon under Workbench lifecycle with
  explicit authority/principal/session/role facts, create/open recovery, the
  ordinary public client and clean signal drain. Workbench retains no grant,
  clock, authentication or authority policy.

## P2 — operational depth

- [x] Moira: carry bounded selected-saga effect evidence and operator receipts
  through the same read-only component in standalone and Workbench interactive
  paths.
- [x] Moira: keep reconciliation mutation guarded until exact confirmation,
  current authority, revision, durable receipt, and outcome-unknown recovery
  are public and qualified end to end.
- [x] Hermes: carry a read-only cursor observation plus exact opaque stream and
  consumer selectors through standalone and Workbench without acquiring the
  acknowledgement-capable consumer facade.
- [x] Hermes: add exact-confirmed event disclosure through a separate read-only
  facade, bounded to 16 events and the negotiated frame ceiling, with explicit
  retention/publication gaps and no global stream or consumer catalogue.
- [x] Hermes: reuse the peer-owned same-connection consumer guard for one
  exact-confirmed acknowledgement in standalone and Workbench, fencing stale
  cursors and never retrying an uncertain mutation.
- [x] Hermes: reuse the peer-owned exact-stream-state guard for one compaction
  in standalone and Workbench, keeping operator authority out of the reusable
  component and selecting any hosted operator authority explicitly at the
  Hermes process boundary.
- [x] Aegis: reuse the peer-owned exact-confirmation/current-epoch guard for
  one-attempt epoch revocation in standalone and Workbench, preserving
  `outcome_unknown` without retry.
- [x] Aegis: reuse the owner-supplied bounded administration-session protocol
  for named issue/attenuate/authorize/list/close in standalone and Workbench;
  grant handles remain session-local and private to the shared manager.
- [x] Aegis: qualify explicit disconnect/reconnect abandonment and deliberate
  rebuild under a new operation name in both standalone and Workbench shells.
  Durable grant replay and lost-reply reconciliation remain nonclaims.
- [x] Aegis: preserve bounded redacted grant/decision pagination through the
  same read-only component in standalone and Workbench interactive paths.
- [x] Chronos: reuse its public Hub and owner service for task creation,
  receipted cancellation, scheduling enablement, global/task pause, capacity,
  and not-before controls through Workbench; prove clean drain and exact
  independent durable reopen without importing mutation semantics.
- [x] Daimon: reuse exact configured agent-session/provider evidence through
  standalone and Workbench without exposing a session catalogue or mutation
  client.
- [x] Daimon: add guarded prompt, advance, permission, cancellation, close and
  reattach only with
  current authority, exact confirmation, durable reconciliation and no-retry
  qualification.
- [x] Daimon: close the integrated hosted lifecycle gate across the production
  owner process, ordinary public client, clean drain and durable reopen.
- [x] Chronos: qualify the selected task and scheduling mutation profile through
  the same public Hub and owner service in standalone and Workbench paths,
  preserving peer-owned receipts, clean drain and exact durable reopen.
- [x] Vellum: qualify the implemented `workspace.edit.v1` and
  `workspace.save.v1` path in standalone and Workbench. Real-owner gates
  preserve revision witnesses, dirty/clean/indeterminate projection, save
  authority refusal, exact persisted bytes and exact replay after a deliberately
  discarded accepted reply. Newer durable operations remain a separate adapter
  selection item.
- [x] Agora: qualify selected `rooms.create.v2` through one owner-client guard
  in standalone and Workbench. The command binds the exact room facts and
  current revision, refuses an inexact confirmation before connection, returns
  the durable owner receipt, and resolves a discarded reply only by replaying
  the identical retained request identity. Broader Agora mutations remain
  unclaimed rather than becoming generic Workbench commands.

## P3 — cross-engine completion

- [x] Record Hermes append at its actual owner boundaries rather than inventing
  a Workbench command: Moira qualifies its workflow-event schema and
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
  same transport-neutral client for standalone and Workbench use, preserves
  pre-send versus outcome-unknown evidence, and keeps its Dispatch/InspectAgora
  decisions separate from Agora's owner-side Execute/Observe enforcement.
  The isolated host qualification also returns this exact client from
  `NativeHost` and proves publish/observe under Workbench lifecycle with
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
- [ ] Complete Moira's process-separated Agora/Vellum workflow and every-boundary
  lost-reply matrix. The Agora response segment is now closed independently:
  Agora retains a caller-stable actor operation, its durable daemon commits one
  addressed response in a separate process, Moira discards the successful reply,
  and a fresh authenticated public client observes the exact result without
  republishing. The Vellum segment is now independently closed as well: its
  authenticated daemon commits one revision-bound durable edit, loses the reply,
  and a fresh public client performs historical lookup without execution
  fallback. The composed Agora→Vellum journey and every-effect fault matrix
  remain open.
- [ ] Complete physical authority/transport/revocation composition for Chronos
  and Vellum federation; finish immutable release qualification for Agora
  federation.
- [ ] Install Vellum's newer durable operations only through their selected
  authority and platform adapters.

## P4 — distribution and additional presentation

- [x] Capture immutable peer and engine revisions and rerun the full standalone,
  attach and host matrix.
- [ ] Close source/legal review, upgrade/migration, supported-platform and
  artifact gates for the selected local deployment.
- [x] Qualify the Workbench GUI local dependency closure across clean committed
  Chronos/Vellum engines and GUI peers.
- [ ] Qualify Workbench GUI artifact distribution, signing and notarization
  before adding peers beyond Chronos and Vellum.
- [ ] Revisit remote deployment, signing/notarization and wider GUI coverage only
  after the local product graph is release-qualified.

## Exit evidence for every claimed cell

For each claimed cell record the exact source state, public client, capability
identifier, topology, authority inputs, real-service scenario, refusal and
outcome-unknown behavior, shutdown result and explicit nonclaims. A passing
read cell cannot close a write cell, and an attached cell cannot close a hosted
cell.
