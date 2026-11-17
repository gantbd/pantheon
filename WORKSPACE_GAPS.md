# Pantheon integration gaps

This ledger records presentation boundaries that the Pantheon shell can name
but cannot yet compose. A gap is not treated as passing evidence and does not
authorize Pantheon to import an engine or recreate its client semantics.

| Surface | Available boundary | Missing boundary | Current fallback |
| --- | --- | --- | --- |
| Agora | Native protocol, service, clients and wire; engine-owned service-free public Unix client for conversation and scheduled-publication profiles; independent read-only `agora-tui` with compatibility client facade; exact-confirmed, revision-bound `rooms.create.v2` guard reused by standalone and Pantheon; explicit conversation-only and routed-composition transports; real Aegis-backed standalone create/publish/observe/drain/reopen; Pantheon-supervised exact process in headless and interactive modes; owner-side identity, time, bootstrap, grant provisioning, durability and cleanup receipts; stale-command, refused-cleanup, abrupt-stop, outcome-unknown, exact-replay, continued-drain, exact-residue-rotation and reopen cuts | Immutable/release provenance; broader domain mutations remain deliberately unclaimed until peer-selected | Attach either explicit transport or supervise the exact standalone binary from its opaque owner config; invoke selected mutations only through Agora's guard and public client; Pantheon never interprets authority facts |
| Moira | Native read/reconciliation clients, peer-owned reconnectable Unix saga port, sovereign durable saga owner, and independent `moira-tui` saga inspection/reconciliation component; standalone and Pantheon census plus bounded selected-saga evidence/operator receipts; qualified Pantheon supervision in both product modes; shared exact-confirmation and durable-recovery guard over an explicitly selected production Aegis owner profile | No selected saga-profile gap; Moira's newer agent, participation and deliberation routes are qualified peer capabilities but explicitly uncomposed by Pantheon; broader process-separated Agora/Vellum workflow qualification remains open | Attach or host the saga peer and invoke the same owner guard; retain attempt receipt inspection through Chronos; never infer reconciliation authority or agent-route support from local identity, supervision, or Moira's package-wide manifest |
| Daimon | Execution/status and agent-session clients; local daemon and supervisor census; independent `daimon-tui` redacted presence plus exact configured session/event evidence; exact-confirmed current-state-fenced active-turn advance/cancellation, ready-session close, detached-session reattach, prompt dispatch and permission decisions reused by standalone and Pantheon; qualified production owner hosting with durable session reopen | Broader owner-side provider/platform qualification and immutable release provenance | Attach the read-only presence peer or host the sovereign production agent application with an exact agent selection; invoke bounded advance, cancel, close, reattach, prompt or permission decision through owner guards; retain attempt inspection through Chronos |
| Grounding | Dendrite typed authenticated `grounding.observe.v1` endpoint alongside MCP; independent `tellus` with bounded provenance/freshness projection | No selected-profile composition gap; surface/read DTOs are deliberately unselected until Dendrite owns their disclosure contract, and Pantheon hosting and mutation are intentionally not applicable | Attach the typed read-only Grounding peer; Dendrite retains its IDE/agent bridge lifecycle and MCP remains agent-facing only |
| Aegis | Administrative/read clients, engine-owned service-free public Unix client, standalone owner daemon, independent `aegis-tui` bounded status with compatibility client facade, navigable redacted pages, exact-confirmed epoch revocation, shared long-lived named grant management, qualified reconnect/rebuild, and Pantheon supervision | No selected-profile session-parity gap; durable grant replay and lost-reply reconciliation are explicit nonclaims | Attach or supervise the read-only peer; invoke the same owner-side bounded administration protocol with exact confirmations in standalone or Pantheon |
| Hermes | Ledger/operator clients plus engine-owned service-free public Unix client; durable local adapter; OS-peer authentication; explicit observer/exact-stream operator daemon and least-privilege exact-stream delivery daemon; independent content-free observation/cursor peer plus exact-confirmed bounded disclosure, acknowledgement and compaction guards; Pantheon-owned headless/interactive create/reopen and operator-command lifecycle; qualified Moira workflow-event producer plus deployed Agora minimized-wake local application and durable receipt endpoint | No presentation-parity mutation gap. Remote provider endpoints and cross-realm revocation remain adapter/deployment work, not Pantheon capability gaps | Attach or host the same public peer; select operator authority explicitly at the Hermes boundary; keep every producer envelope and delivery policy in its producer or cross-engine adapter, never Pantheon |

Federation peers and cross-engine adapters remain subordinate to their owning
engine or workflow presentation. Keystone remains presentation-free.

Pantheon has deployment-owned typed lifecycle registration seams for the
formerly attach-only peers. Those seams return each owner's concrete client
unchanged. Hermes, Aegis, Daimon, Agora, and Moira now supply profile-driven
sovereign process hosts; Grounding hosting is deliberately
not applicable.

## Current qualification state

The reusable/CLI boundary and versioned headless contract are now aligned for
Agora, Chronos, Vellum, Moira, Daimon, Hermes, Aegis, Grounding, and Pantheon. Each standalone CLI names its public
read capability and constructs no TUI state in headless mode. Pantheon attaches
the same clients and preserves healthy observations beside a failed peer.
Each peer also owns a closed presentation capability manifest; Pantheon
uses those declarations as source material but negotiates bounded profile
requirements against a Pantheon-owned exact route catalog. Mode, topology,
selectors and launch features determine the delivered route before any
connection. Separate guarded commands cannot satisfy an aggregate profile
requirement, and a route match is never service authorization.

For the initial single-user/local-machine management scope, the headless
aggregate also publishes readiness at observation time, revision where exposed,
certainty and fencing, attached versus hosted lifecycle, the peer-owned
read-only authority boundary, and exact hosted drain dispositions. It leaves
unknown revisions null, does not treat attachment as health, and requires a
restart for topology reconfiguration. Live reconfiguration, fleet management,
and graphical management surfaces remain later scope. One bounded local
operation identifier correlates the observation, management, and drain evidence
without being forwarded as identity, authority, or idempotency.
`reconfigure-plan` compares two validated profiles without connecting and
machine-classifies shell-only versus full-local-topology restart. Embedding hosts can execute an
exact-peer restart with a typed stop/start receipt; refused or uncertain stops
fence further changes and remain visible during final drain. Pantheon does not
mutate a live profile in place. The standalone management session now preflights
the exact next profile, drains the current topology, emits correlated receipts,
and replaces its own process on the retained stdio channel; refusal or
uncertainty fences that handoff. Embedding hosts can apply the host portion of a
validated full-topology plan with typed client
retention, correlated old/new receipts, atomic replacement sealing, and partial
replacement rollback.
The same live host exposes a typed in-process status snapshot containing
lifecycle, hosted peers, reconfiguration eligibility and retained restart
receipts. A bounded long-running `pantheon manage` stdio session exposes status,
fresh observation and drain to a single-user parent supervisor without opening
a socket or gaining peer authority. No remote or ambient-authority endpoint is
claimed.

| Core peer | Standalone interactive | Standalone headless | Pantheon attach | Pantheon host |
| --- | --- | --- | --- | --- |
| Agora | Reused TUI; real-daemon room/message journey plus real Aegis-backed durable process | `rooms.read.v2`; real daemon | Interactive + headless over explicit conversation or composition transport | Exact sovereign process; real Aegis, public client, reverse drain, and durable create/open qualified in both modes |
| Chronos | Reused TUI; real-service read and selected guarded-write journeys | `tasks.observe.v1`; real service; partial workstreams | Interactive + headless; real service; selected guarded-write profile with exact durable reopen | Native owner; interactive + headless client path; joined drain; selected guarded-write profile with exact durable reopen |
| Vellum | Reused TUI; real-daemon `workspace.edit.v1`/`workspace.save.v1` journey with witnessed edits, dirty/clean/indeterminate projection and exact uncertain replay | `workspace.inventory.v1`; real daemon | Interactive + headless; real daemon; same reusable editor and save refusal | Native owner; bounded roots/auth/action ceiling; authorized exact persistence, denied-save unchanged source, and joined drain |
| Moira | Reused saga TUI; real-daemon census/evidence/receipt and exact reconciliation journeys | Aggregate `sagas.inspect.v1`; reconciliation confirmation/recovery and newer agent/participation/deliberation controls remain separate peer routes | Interactive saga evidence/receipt component + aggregate census; separately invoked shared reconciliation guards; real daemons | Sovereign `moira-saga-local`; opaque Aegis profile, durable mutation/receipt reopen and clean drain qualified |
| Daimon | Reused redacted presence plus exact configured agent-session TUI; separate guarded advance, cancel, close, reattach, prompt and permission commands | Aggregate selected presence/session observation; guarded mutations remain separate commands | Presence and selected-agent projections render together when both are configured; aggregate headless preserves both; same owner mutation guards and strict operator documents | Qualified sovereign supervised agent host in both modes with real Aegis/Chronos admission, durable session reopen and clean drain |
| Hermes | Reused ledger TUI plus owner guards | Aggregate `ledger.observe.v1` and selected cursor observation; acknowledge/compact/disclose remain separate guarded commands | Interactive component + aggregate headless; exact-confirmed owner commands are independently classified | Observer/operator owner process; create/reopen, guarded mutation and clean drain qualified in both modes |
| Aegis | Reused read-only authority TUI plus peer-owned authority guards | Aggregate `authority.observe.v1`; revoke/attenuate/admin-session remain separate guarded commands | Interactive/read-only aggregate plus independently classified exact-confirmed owner commands and long-lived named manager; grant handles never escape | Owner daemon, managed session, disconnect abandonment and deliberate rebuild qualified; replay/reconciliation remain nonclaims |
| Grounding | Reused typed Tellus | `grounding.observe.v1`; real Dendrite + Rune fixture | Interactive component + aggregate headless; real service | Not applicable; attach to independently deployed Dendrite |

This is qualified immutable local-source evidence, not a remotely reachable
release qualification. The eight peer TUI and eight owner source revisions are
pinned in Pantheon's source receipt and clean; the archived qualifier rebuilds
that exact graph and seeds Vellum's declared Keystone Git identity from the
same pinned local object before entering offline mode. Remote reachability is
unqualified. Agora's production-shaped standalone and
supervised process boundaries are locally qualified, but immutable provenance
is not. The joint Chronos/Vellum hosted graph is locally qualified. The
all-three in-process graph is locally qualified with an injected explicitly
test-only Agora authority factory. The selected production-shaped three-peer
topology is also locally qualified by supervising Agora's exact sovereign
process; immutable release qualification remains open. Agora's daemon
requires its application to bind each accepted connection to an exact
participant, verified subject, Aegis session/grant, and trusted time. Aegis has
a qualified Unix peer authenticator and Unix client port, but neither boundary
defines that deployment mapping; Pantheon must not invent it. Chronos
standalone resolves a vendored
`keystone-storage` package while Agora's daemon resolves a sibling package with
the same compatible checkpoint/commit subset, name, and version. Cargo refuses both
identities in one ordinary lockfile. The nested qualification derives a scoped
source patch at runtime and now proves one process can own and reverse-drain all
three peers from one lock closure without changing standalone source policy.
Pointing Chronos directly at the sibling checkout was tested and rejected by
its no-outward-path, relocatable source-profile gate. That constraint belongs
only to the test-only in-process topology. The selected production topology
supervises Agora as a sovereign process and keeps Chronos native, so their Cargo
source identities never enter one lockfile. Any future in-process product must
still use one immutable remotely retrievable Keystone revision or published
artifact rather than a workspace-only path assumption.
The mixed deployment that owns Chronos and Vellum while attaching Agora is also
qualified across all three interactive panes and one aggregate headless report.
The separate macOS Pantheon GUI now has a clean committed local dependency
closure across the Chronos/Vellum engines and their GUI peers, plus a qualified
unsigned prepared artifact with an exact legal and checksum closure. GUI
distribution, signing, notarization and remote immutable reachability are
explicitly deferred outside the selected local delivery and are not inferred
from that prepared artifact.

## Future reopening conditions

The selected local standalone-and-Pantheon program has no remaining delivery
items. The following broader work stays outside its claims and must be reopened
by a concrete owner or product profile rather than absorbed into Pantheon:

1. Promote Agora's now-qualified real standalone and Pantheon-supervised
   process journeys to immutable release evidence. Process drain remains owner-side. Deployment continues
   to supply credential mapping, authority epoch, lifetime, resource, and
   subject unchanged through the opaque Agora config.
2. Retain Moira's qualified exact-confirmation, current-authority, revision,
   durable-receipt and outcome-unknown recovery guard. Compose its separate
   agent, participation or deliberation routes only through their owning public
   clients and components; until then they remain machine-classified as absent
   from Pantheon. Broader process-separated Agora/Vellum workflows stay with
   their owners.
3. Retain Daimon's qualified exact-session evidence, bounded active-turn
   advance, cancellation, close, reattach, prompt-dispatch and
   permission-decision paths plus qualified sovereign hosting; keep broader
   provider, credential and platform qualification owner-side.
4. Reopen typed Dendrite surface/read DTOs only after Dendrite owns their exact
   content disclosure, provenance, freshness, and bounds; MCP remains outside
   Pantheon's internal protocol.
5. Keep Hermes payload disclosure separate from routine observation and retain
   its exact selector and bounds. Do not add generic append to Pantheon:
   Moira's qualified workflow-event publisher and the qualified Agora minimized-
   wake adapter own their schemas and exact replay law. Close Agora endpoint and
   deployment policy in that adapter, not in Hermes presentation.
6. Reopen GUI signing, notarization, distribution, wider-platform coverage, or
   remote delivery only with a concrete product decision and the required
   credentials, deployment policy, and support profile.
