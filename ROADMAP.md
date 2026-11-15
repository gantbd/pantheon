# Workbench terminal and host roadmap

Workbench is a product composition root, not a new semantic engine. It may own
one executable invocation, process lifecycle, terminal, profile, and deployment
topology while every peer continues to own its protocol, authority, state,
effects, refusals, and uncertainty.

## Package boundaries

The target workspace separates these responsibilities:

- `workbench-tui`: reusable Ratatui shell and peer-presentation composition. It
  owns layout, focus, global navigation, and typed shell lifecycle requests. It
  owns no argument parsing, terminal setup/restoration, daemon startup, process
  exit, or peer domain behavior.
- Workbench CLI/application: the standalone composition root. It parses the
  versioned profile, chooses interactive or headless mode, selects attachment or
  hosted topology, establishes authority inputs, and owns process shutdown.
- Workbench host/service: optional composite lifecycle owner. It starts and
  supervises selected peer service runtimes and constructs their public clients.
  It does not introduce a universal semantic protocol or shared authoritative
  store.

Peer TUI libraries are dependencies of `workbench-tui`. Peer standalone CLI
shells are not. A peer close request is data: Workbench may remove its surface,
whereas that peer's standalone CLI may interpret the same request as exit.

## Deployment modes

1. **Attach:** connect to independently running peer services.
2. **Hosted:** one Workbench executable invocation starts selected peer service
   runtimes in-process where supported, constructs their clients, and composes
   their TUI components.
3. **Supervised:** when a runtime cannot be linked into the host, one Workbench
   invocation supervises the peer process and connects through its native typed
   protocol. The profile must expose this distinction; supervision is not
   semantic embedding.
4. **Headless:** construct the same hosted or attached service/client graph
   without a terminal or presentation state. Bounded commands, snapshots,
   health, and orderly shutdown use peer-owned client contracts.

A future Workbench daemon may expose one operational endpoint for lifecycle and
discovery, but semantic requests remain routed to versioned peer protocols. One
daemon must not become one untyped command bus or one merged authority domain.

## Delivery order

### 1. Freeze the reusable shell

- Remove remaining CLI and terminal assumptions from the TUI library boundary.
- Keep all peer components optional and render exact missing-capability states.
- Define typed lifecycle, focus, subscription, and background-work outputs.
- Preserve each peer's confirmation and outcome-unknown fences when unfocused.

Exit check: the shell renders and routes synthetic peers without parsing
arguments, opening a terminal, connecting a service, or terminating a process.

### 2. Extract the Workbench CLI

- Move profile parsing, terminal lifecycle, signal handling, and exit codes into
  a separate application package/binary while retaining compatible invocation.
- Add explicit `interactive` and `headless` commands; do not infer headless mode
  from the absence of a terminal.
- Keep credentials and grant handles outside serialized workspace profiles.

Exit check: one profile can run the interactive shell or a headless snapshot
against the same attached clients with identical authorization and freshness.

Status: the current CLI has explicit `interactive` and `headless` commands
(with `--headless` and `--snapshot` compatibility aliases). Headless emits a versioned JSON
report for any configured subset of Agora, Chronos workstreams, and Vellum,
uses their public clients directly, and refuses aggregate success on partial
failure. The executable, profile parser, terminal lifecycle, and hosted-service
helpers now live in the separate `workbench-tui-cli` package; the root package
contains the reusable shell and peer-presentation composition only.
Profiles may require bounded peer-owned capability identifiers. Workbench
negotiates them against the selected mode before any connection while leaving
runtime authorization and refusal with the owning service.

### 3. Add composite service hosting

- Define a host registry whose entries construct peer services and clients from
  typed configuration. Start with Agora, Chronos, and Vellum.
- Give every hosted peer an independent state root, recovery policy, limits,
  authority adapter, and shutdown receipt.
- Refuse partial startup safely and report partial shutdown or uncertain effects
  without claiming a clean aggregate result.

Exit check: one Workbench invocation hosts all three services, drives their
reused TUIs, and shuts them down in a bounded order without an external daemon.

Status: the CLI package now contains a typed-client host registry. It returns
each peer's concrete client to its owning integration, retains only operational
shutdown callbacks, refuses duplicate/post-seal starts, drains in reverse order,
preserves refused or outcome-unknown shutdown receipts, and rolls back partial
assembly with exact receipts when a guarded factory start fails. Concrete
Agora, Chronos, and Vellum service factories plus a hosted CLI/profile mode
remain open.

### 4. Qualify headless parity

- Run the same hosted topology without Ratatui, focus, selection, or cursor state.
- Provide bounded per-peer command and snapshot routing plus aggregate health;
  keep semantic results typed by the owning peer.
- Exercise restart, one-peer failure, lost replies, cancellation, and shutdown.

Exit check: interactive and headless journeys produce the same peer-owned
receipts/refusals for equivalent operations and never infer aggregate success
from presentation or process state.

Status: attached read parity is qualified for Agora, Chronos, and Vellum,
including one unavailable Chronos workstream alongside healthy peers. Hosted
service parity, mutation routing, restart, and partial-shutdown qualification
remain open.

### 5. Extend peer coverage

Integrate Moira, Daimon, Grounding/Dendrite, Aegis, and Hermes only after each
has a public client plus a reusable TUI component. Track unavailable boundaries
in `WORKSPACE_GAPS.md`; do not reproduce peer semantics inside Workbench.

## Qualification matrix

Every peer integration must pass:

| Owner | Interactive | Headless |
| --- | --- | --- |
| Peer standalone CLI | Reused peer TUI | Peer client commands without UI state |
| Workbench attach | Reused peer TUI over remote client | Same remote client graph |
| Workbench host | Reused peer TUI over hosted client | Same hosted service/client graph |

The matrix includes authorization denial, stale revision, service unavailable,
outcome unknown, recovery, and orderly/partial shutdown cases.

Current status: the standalone and Workbench-attach read cells pass for Agora,
Chronos, and Vellum with explicit capability IDs. Agora's passing evidence uses
the selected private client against a bounded protocol fixture; Chronos and
Vellum use their real services. The Workbench-host row remains open for all
three and is not implied by attachment success.
