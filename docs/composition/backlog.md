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
- [x] Keep Hermes hosting out of the mixed engine graph: Workbench now owns the
  owner-supplied standalone daemon process and attaches the same public client,
  preserving Hermes authentication, authorization, storage and shutdown.
- [ ] Converge Chronos and Agora hosting on one immutable Keystone source
  identity so their normal in-process product host graph needs no
  qualification-only patch. Use a
  remotely retrievable pinned Git revision or published artifact; a sibling
  path breaks Chronos's relocatable source-profile gate, while separate vendored
  paths cannot share one Cargo lock identity.

## P1 — production composition

- [ ] Supply Agora's production peer-provenance, Aegis grant/time and durable
  owner factory without putting credential policy in Workbench.
- [x] Define a typed Grounding presentation client over Dendrite's bounded
  observation/provenance behavior, separate from MCP.
- [x] Add `grounding-tui` and its standalone interactive/headless application,
  then attach the same component to Workbench.
- [ ] Extend Grounding with selected surface/read DTOs only after exact content
  disclosure, provenance, freshness and response bounds are fixed.
- [x] Add deployment-owned typed lifecycle factory seams for Moira, Daimon,
  Hermes, Aegis, and Grounding while preserving separate state, authority,
  recovery and shutdown receipts.
- [ ] Promote the remaining seams to standalone native or supervised profile factories
  only after each owner supplies explicit storage, authentication, recovery and
  shutdown composition and passes real lifecycle qualification.
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
- [ ] Moira: keep reconciliation mutation guarded until exact confirmation,
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
- [ ] Daimon: after active work settles, exact execution/provider evidence and
  guarded dispatch/resume with no-redispatch qualification.
- [ ] Agora, Chronos and Vellum: add remaining mutations only where standalone
  and Workbench paths preserve identical peer-owned confirmation and receipts.

## P3 — cross-engine completion

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
  remains part of the P1 Agora provenance/grant/time/durability item.
- [ ] Close the Agora–Hermes concrete endpoint and deployed production profile.
  The adapter-owned public-Aegis-client authority wiring is qualified against
  the real Aegis daemon and public Unix client, including live epoch revocation
  fencing a previously valid grant; the deployment must still select peer
  provenance, production session/grant issuance, trusted time source, endpoint
  receipts/network policy, and durable owner lifecycle.
- [ ] Complete Moira's process-separated Agora/Vellum workflow and every-boundary
  lost-reply matrix.
- [ ] Complete physical authority/transport/revocation composition for Chronos
  and Vellum federation; finish immutable release qualification for Agora
  federation.
- [ ] Install Vellum's newer durable operations only through their selected
  authority and platform adapters.

## P4 — distribution and additional presentation

- [ ] Capture immutable peer and engine revisions and rerun the full standalone,
  attach and host matrix.
- [ ] Close source/legal review, upgrade/migration, supported-platform and
  artifact gates for the selected local deployment.
- [ ] Qualify Workbench GUI dependency closure and distribution before adding
  peers beyond Chronos and Vellum.
- [ ] Revisit remote deployment, signing/notarization and wider GUI coverage only
  after the local product graph is release-qualified.

## Exit evidence for every peer

For each claimed cell record the exact source state, public client, capability
identifier, topology, authority inputs, real-service scenario, refusal and
outcome-unknown behavior, shutdown result and explicit nonclaims. A passing
read cell cannot close a write cell, and an attached cell cannot close a hosted
cell.
