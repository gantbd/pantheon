# Workbench integration gaps

This ledger records presentation boundaries that the Workbench shell can name
but cannot yet compose. A gap is not treated as passing evidence and does not
authorize Workbench to import an engine or recreate its client semantics.

| Surface | Available boundary | Missing boundary | Current fallback |
| --- | --- | --- | --- |
| Agora | Native protocol, service, client and wire; independent read-only `agora-tui`; selected private Unix client with endpoint ownership checks, bounded handshake/framing and truthful transmission uncertainty; real local daemon lifecycle qualified through a deployment-owned factory | Production peer-provenance, Aegis grant/time, and durable-host factory for the standalone Workbench profile | A profile can attach an existing socket; a trusted host can inject the authority-bearing peer |
| Moira | Native read/reconciliation clients and local daemon; independent `moira-tui` saga inspection/reconciliation component; standalone and Workbench headless observation | Guarded operator reconciliation actions and production Workbench host factory | Attach the read-only peer; retain attempt receipt inspection through Chronos |
| Daimon | Execution/status clients, local daemon and supervisor census; independent `daimon-tui` redacted presence peer | Exact-ID execution detail, guarded dispatch/resume, and production Workbench host factory | Attach the read-only presence peer; retain attempt inspection through Chronos |
| Grounding | Dendrite MCP service over authored Rune material | Typed presentation client and composable Ratatui peer | Explicit unavailable surface |
| Aegis | Administrative/read clients and local daemon | Guarded authority-administration Ratatui peer | Explicit unavailable surface |
| Hermes | Ledger/operator clients and local daemon | Ledger operations Ratatui peer | Explicit unavailable surface |

Federation peers and cross-engine adapters remain subordinate to their owning
engine or workflow presentation. Keystone remains presentation-free.

## Current qualification state

The reusable/CLI boundary and versioned headless contract are now aligned for
Agora, Chronos, Vellum, Moira, Daimon, and Workbench. Each standalone CLI names its public
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
| Moira | Reused saga TUI; public-port rendering gate | `sagas.inspect.v1`; real SQLite daemon | Interactive component + aggregate headless; real daemon | Open; attach only |
| Daimon | Reused redacted presence TUI; public-port rendering gate | `agents.presence.v1`; real durable daemon and roster | Interactive component + aggregate headless; real daemon | Open; attach only |

This is qualified local working-tree evidence, not an immutable release
qualification. The three peer TUI commits are exact and clean in Workbench's
source receipt, but several engine trees differ from their recorded revisions,
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
The mixed deployment that owns Chronos and Vellum while attaching Agora is also
qualified across all three interactive panes and one aggregate headless report.

## Reconciliation order

1. Converge Chronos and Agora on one Keystone source identity so the already
   qualified all-three owner graph builds without a local source patch.
2. Supply Agora's production peer-provenance, Aegis grant/time, and durable-host
   factory without moving those authority semantics into Workbench.
3. Extend Moira's read-only trace with census, evidence, and operator receipts;
   expose reconciliation actions only with exact confirmation, current authority,
   durable receipts, and outcome-unknown recovery.
4. Extend Daimon's peer from redacted roster observation to exact-ID execution
   detail; add dispatch/resume only with no-redispatch and uncertainty evidence.
5. Define a typed Dendrite presentation boundary rather than making MCP the
   Workbench's canonical internal protocol.
6. Add Aegis and Hermes operator peers only with their existing authorization,
   uncertainty and content-minimization laws intact.
