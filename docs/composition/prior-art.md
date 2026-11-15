# Axon and Engram prior-art ledger

Axon and Engram are design, implementation and failure-history sources for
Workbench. They are not peers Workbench must integrate, and their state or
protocols do not become current authority. Adoption means restating a useful
pattern against the owning modern peer and qualifying it there.

## Axon lessons

| Prior-art capability or lesson | Destination owner | Adoption rule |
| --- | --- | --- |
| One headless semantic path used by every UI tier | Each peer client/service; Workbench aggregate headless shell | Preserve peer protocols and failure meaning; do not recreate one Axon-shaped universal API. |
| TUI model/message/update discipline, buffers, focus and global navigation | Reusable peer TUIs for local state; Workbench for global routing | Reuse interaction lessons, not Axon runtime or room state. |
| Joined client/server tracing, readiness and content-safe operational state | Each peer for semantic signals; Workbench overview for aggregation | Trace correlation crosses boundaries; recovery and authority remain peer-owned. |
| Connection is distinct from participant and authority | Agora plus deployment authentication and Aegis | Workbench profiles may name claims, never credentials or grants. |
| Durable confirmation, outcome-unknown fencing and reconnect failure history | Owning peer clients and adapters | Preserve exact operation identity; never translate timeout into failure or retry. |
| Workspace/room/conversation interaction vocabulary | Workbench navigation, Agora rooms, Vellum workspaces and Chronos workstreams | Adopt useful navigation without restoring Axon's merged authority model. |
| Git, editor, shell and process operator workflows | Vellum for documents; Daimon for bounded execution; an explicit future VCS owner for repository mutation | Do not place ambient filesystem, Git or process authority in Workbench. |
| Permission prompts, approvals and capability-aware actions | Aegis decisions plus command-owning peer confirmations | A UI confirmation is not authorization; each effect retains its owner and receipt. |
| Multiple connections observing one evolving product | Peer observation contracts and Workbench projections | Subscription freshness, revocation and backpressure must be explicit per peer. |

Primary evidence is in the workspace's `axon/arch/ARCHITECTURE.md`,
`axon/req/REQUIREMENTS.md` and `axon/plan/ROADMAP.md`.

## Engram lessons

| Prior-art capability or lesson | Destination owner | Adoption rule |
| --- | --- | --- |
| Server rail and unified shell navigation | Workbench shell | Surfaces select sovereign peers; the rail does not create a federated authority. |
| Editor/read lens and artifact navigation | Vellum presentations | Preserve source, revision, persistence and edit authority in Vellum. |
| Agenda, Kanban and locality-scoped task views | Chronos projections, optionally linked from Vellum artifacts | Cross-links are typed identities/adapters, not shared mutable task/document state. |
| Chat, operations channels and run threads | Agora for conversation; Moira for workflow; peer health in Workbench overview | Do not use a chat transcript or operations feed as workflow evidence. |
| Agent run cockpit, progress, interruption and durable resume | Moira coordination plus Daimon execution/provider evidence | Process liveness and provider output cannot establish workflow completion. |
| Worktree admit/discard and adjudication | Explicit future repository/VCS boundary, authorized by its owning workflow | Workbench may present receipts but must not acquire ambient Git authority. |
| Provider/model settings and rosters | Daimon provider profiles, Aegis identities/authority and Workbench secret-free deployment profiles | Credentials remain deployment inputs and never enter serialized Workbench profiles. |
| Headless one-shot invocation | Each standalone peer shell and Workbench headless aggregation | Deterministic exit and versioned output are retained without fabricating UI state. |
| Crash recovery and supervised long-lived sessions | Owning services and application shells | Recovery resumes only from durable peer evidence, not a generic Workbench process record. |
| Grounded development and skill/FSM discovery | Dendrite typed presentation client and Grounding peer | MCP remains an agent projection; human presentation gets a bounded typed client. |

Primary evidence is in the workspace's
`engram/docs/grounded-roadmap-tasks.md` and
`engram/docs/GROUNDED-ROADMAP.md`.

## Admission questions

Before carrying a prior-art idea into current code, answer:

1. Which current peer owns the authoritative state and effect?
2. Is the value a semantic capability, a presentation pattern or only a lesson
   from a historical failure?
3. What public client contract supports standalone and Workbench paths?
4. What confirmation, authorization, revision and uncertainty evidence must the
   presentation preserve?
5. Which legacy coupling is intentionally not reproduced?
6. What focused and real-service evidence makes the adoption more than naming?

Items without an owner remain research notes. They do not default to Workbench,
Moira or Keystone.
