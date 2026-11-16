# Workbench integration gaps

This ledger records presentation boundaries that the Workbench shell can name
but cannot yet compose. A gap is not treated as passing evidence and does not
authorize Workbench to import an engine or recreate its client semantics.

| Surface | Available boundary | Missing boundary | Current fallback |
| --- | --- | --- | --- |
| Agora | Native protocol, service, client and wire; independent read-only `agora-tui`; selected private Unix client with endpoint ownership checks, bounded handshake/framing and truthful transmission uncertainty; real local daemon lifecycle qualified through a deployment-owned factory; Agora–Hermes now supplies the concrete public-Aegis-client authority adapter with fresh trusted-time input | Deployment selection of peer provenance, Aegis session/grant/time source, concrete endpoint, and durable-host factory for the standalone Workbench profile | A profile can attach an existing socket; a trusted host can inject the authority-bearing peer |
| Moira | Native read/reconciliation clients and local daemon; independent `moira-tui` saga inspection/reconciliation component; standalone and Workbench census plus bounded selected-saga evidence/operator receipts | Guarded operator reconciliation actions and production Workbench host factory | Attach the read-only peer; retain attempt receipt inspection through Chronos |
| Daimon | Execution/status clients, local daemon and supervisor census; independent `daimon-tui` redacted presence peer | Exact-ID execution detail, guarded dispatch/resume, and production Workbench host factory | Attach the read-only presence peer; retain attempt inspection through Chronos |
| Grounding | Dendrite typed authenticated `grounding.observe.v1` endpoint alongside MCP; independent `grounding-tui` with bounded provenance/freshness projection | Selected surface/read navigation and production Workbench host factory | Attach the typed read-only Grounding peer; MCP remains agent-facing only |
| Aegis | Administrative/read clients, standalone owner daemon, independent `aegis-tui` bounded status, navigable redacted pages, exact-confirmed epoch revocation, shared long-lived named grant management, qualified reconnect/rebuild, and Workbench supervision | No selected-profile session-parity gap; durable grant replay and lost-reply reconciliation are explicit nonclaims | Attach or supervise the read-only peer; invoke the same owner-side bounded administration protocol with exact confirmations in standalone or Workbench |
| Hermes | Ledger/operator clients; durable local adapter; OS-peer authentication; explicit observer/exact-stream operator daemon; independent content-free observation/cursor peer plus exact-confirmed bounded disclosure, acknowledgement and compaction guards; Workbench-owned headless/interactive create/reopen and operator-command lifecycle | Producer-specific guarded append integrations | Attach or host the same public peer; select operator authority explicitly at the Hermes boundary; keep producer envelopes outside Workbench |

Federation peers and cross-engine adapters remain subordinate to their owning
engine or workflow presentation. Keystone remains presentation-free.

Workbench has deployment-owned typed lifecycle registration seams for all five
formerly attach-only peers. Those seams accept an owner factory and return its
concrete client unchanged. Hermes now supplies the first profile-driven
sovereign process host; the other host cells remain open.

## Current qualification state

The reusable/CLI boundary and versioned headless contract are now aligned for
Agora, Chronos, Vellum, Moira, Daimon, Hermes, Aegis, Grounding, and Workbench. Each standalone CLI names its public
read capability and constructs no TUI state in headless mode. Workbench attaches
the same clients and preserves healthy observations beside a failed peer.
Each peer also owns a closed presentation capability manifest; Workbench
negotiates bounded profile requirements against the selected mode before any
connection without treating a manifest match as service authorization.

| Core peer | Standalone interactive | Standalone headless | Workbench attach | Workbench host |
| --- | --- | --- | --- | --- |
| Agora | Reused TUI; real-daemon room/message journey | `rooms.read.v2`; real daemon | Interactive + headless, protocol fixture | Injected factory; real daemon/public-client lifecycle qualified with test-only authority |
| Chronos | Reused TUI; real-service journeys | `tasks.observe.v1`; real service; partial workstreams | Interactive + headless; real service | Native owner; interactive + headless client path; joined drain |
| Vellum | Reused TUI; real-daemon edit/save journey | `workspace.inventory.v1`; real daemon | Interactive + headless; real daemon | Native owner; bounded roots/auth/action ceiling; joined drain |
| Moira | Reused saga TUI; real-daemon census/evidence/receipt journey | `sagas.inspect.v1`; real SQLite daemon | Interactive evidence/receipt component + aggregate census; real daemon | Open; attach only |
| Daimon | Reused redacted presence TUI; public-port rendering gate | `agents.presence.v1`; real durable daemon and roster | Interactive component + aggregate headless; real daemon | Open; attach only |
| Hermes | Reused ledger TUI plus owner guards | `ledger.observe.v1`, `ledger.acknowledge.v1`, `ledger.compact.v1`; real durable selected daemon | Interactive component + aggregate headless; exact-confirmed owner commands | Observer/operator owner process; create/reopen, guarded mutation and clean drain qualified in both modes |
| Aegis | Reused read-only authority TUI plus peer-owned authority guards | `authority.observe.v1`, `authority.revoke-epoch.v1`, bounded `authority.attenuate-authorize-once.v1`, and `aegis.admin-session.v1`; real durable daemon | Interactive/read-only aggregate plus the same exact-confirmed owner commands and long-lived named manager; grant handles never escape | Owner daemon, managed session, disconnect abandonment and deliberate rebuild qualified; replay/reconciliation remain nonclaims |
| Grounding | Reused typed Grounding TUI | `grounding.observe.v1`; real Dendrite + Rune fixture | Interactive component + aggregate headless; real service | Open; attach only |

This is qualified local working-tree evidence, not an immutable release
qualification. The eight peer TUI source revisions are pinned in Workbench's
source receipt and clean, but several engine trees differ from their recorded
revisions,
remote reachability is unqualified, and Agora still lacks a production
standalone factory. The joint Chronos/Vellum hosted graph is locally qualified;
the all-three graph is locally qualified with an injected explicitly test-only
Agora authority factory, while the full production three-peer hosted profile
remains open rather than being inferred from that evidence. Agora's daemon
requires its application to bind each accepted connection to an exact
participant, verified subject, Aegis session/grant, and trusted time. Aegis has
a qualified Unix peer authenticator and Unix client port, but neither boundary
defines that deployment mapping; Workbench must not invent it. Chronos
standalone resolves a vendored
`keystone-storage` package while Agora's daemon resolves a sibling package with
the same compatible checkpoint/commit subset, name, and version. Cargo refuses both
identities in one ordinary lockfile. The nested qualification derives a scoped
source patch at runtime and now proves one process can own and reverse-drain all
three peers from one lock closure without changing standalone source policy.
Pointing Chronos directly at the sibling checkout was tested and rejected by
its no-outward-path, relocatable source-profile gate. Production convergence
therefore requires one immutable remotely retrievable Keystone revision or
published artifact consumed by both owners; it must not replace the current
patch with a workspace-only path assumption.
The mixed deployment that owns Chronos and Vellum while attaching Agora is also
qualified across all three interactive panes and one aggregate headless report.

## Reconciliation order

1. Converge Chronos and Agora on one Keystone source identity so the already
   qualified all-three owner graph builds without a local source patch.
2. Supply Agora's production peer-provenance, Aegis grant/time, and durable-host
   factory without moving those authority semantics into Workbench.
3. Expose Moira reconciliation actions only with exact confirmation, current
   authority, revision, durable receipts, and outcome-unknown recovery; retain
   the now-qualified read-only evidence/receipt path unchanged.
4. Extend Daimon's peer from redacted roster observation to exact-ID execution
   detail; add dispatch/resume only with no-redispatch and uncertainty evidence.
5. Extend the typed Dendrite presentation boundary with selected surface/read
   DTOs only after their content disclosure, provenance, freshness, and bounds
   are fixed; MCP remains outside Workbench's internal protocol.
6. Keep Hermes payload disclosure separate from routine observation and retain
   its exact selector and bounds. Add ledger mutations only with authorization,
   uncertainty, retention and content-minimization laws intact.
7. Keep Aegis issue and attenuation unavailable until a long-lived session can
   preserve its handles and reconcile a lost reply to an exact operation;
   epoch revocation already reuses the qualified exact-confirmation,
   current-epoch and no-retry guard.
