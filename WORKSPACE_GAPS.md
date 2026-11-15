# Workbench integration gaps

This ledger records presentation boundaries that the Workbench shell can name
but cannot yet compose. A gap is not treated as passing evidence and does not
authorize Workbench to import an engine or recreate its client semantics.

| Surface | Available boundary | Missing boundary | Current fallback |
| --- | --- | --- | --- |
| Agora | Native protocol, service, client and wire; independent read-only `agora-tui`; selected private Unix client with endpoint ownership checks, bounded handshake/framing and truthful transmission uncertainty | Packaged standalone daemon executable and real-daemon Workbench journey | A profile can attach an existing socket; a trusted host can inject the peer |
| Moira | Native read/reconciliation clients and local daemon | Workflow trace and guarded reconciliation Ratatui peer | Attempt receipt inspection through Chronos |
| Daimon | Execution/status clients, local daemon and supervisor census | Execution and agent-operations Ratatui peer | Attempt and roster inspection through Chronos |
| Grounding | Dendrite MCP service over authored Rune material | Typed presentation client and composable Ratatui peer | Explicit unavailable surface |
| Aegis | Administrative/read clients and local daemon | Guarded authority-administration Ratatui peer | Explicit unavailable surface |
| Hermes | Ledger/operator clients and local daemon | Ledger operations Ratatui peer | Explicit unavailable surface |

Federation peers and cross-engine adapters remain subordinate to their owning
engine or workflow presentation. Keystone remains presentation-free.

## Current qualification state

The locked Workbench build, component tests, profile tests, and the live Agora
gate pass against the present coordinated working trees. The live gate covers
both host-supplied composition and profile-driven attachment through the Unix
client to a protocol fixture. This is working-tree evidence, not an immutable
release qualification: several engine trees differ from their source receipts,
remote reachability is unqualified, and Agora still lacks a packaged standalone
daemon journey.

## Reconciliation order

1. Qualify the selected Agora client against a packaged real daemon composition.
2. Add a Moira workflow trace before exposing operator reconciliation actions.
3. Promote Daimon attempt and roster inspection into a peer without removing
   the task-local Chronos route.
4. Define a typed Dendrite presentation boundary rather than making MCP the
   Workbench's canonical internal protocol.
5. Add Aegis and Hermes operator peers only with their existing authorization,
   uncertainty and content-minimization laws intact.
