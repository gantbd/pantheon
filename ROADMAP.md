# Workbench terminal and host roadmap

The normalized cross-repository program and Axon/Engram prior-art ledger live in
[`docs/composition/`](docs/composition/README.md). This roadmap remains the
detailed terminal/host implementation record.

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
Explicit headless configuration or capability negotiation failures retain the
versioned aggregate envelope and cannot fall through to interactive startup.

### 3. Add composite service hosting

- Define a host registry whose entries construct peer services and clients from
  typed configuration. Start with Agora, Chronos, and Vellum.
- Give every hosted peer an independent state root, recovery policy, limits,
  authority adapter, and shutdown receipt.
- Refuse partial startup safely and report partial shutdown or uncertain effects
  without claiming a clean aggregate result.

Exit check: one Workbench invocation hosts all three services, drives their
reused TUIs, and shuts them down in a bounded order without an external daemon.

Status: the CLI package now contains a typed-client host registry plus native
Chronos and Vellum owners used by both interactive and headless execution. The registry returns
each peer's concrete client to its owning integration, retains only operational
shutdown callbacks, refuses duplicate/post-seal starts, drains in reverse order,
preserves refused or outcome-unknown shutdown receipts, and rolls back partial
assembly with exact receipts when a guarded factory start fails. An injected
Agora authority factory is qualified against the real daemon and public client;
Agora now owns the single-principal private-endpoint authenticator and
non-regressing Unix-epoch clock plus an adopted bounded fresh-engine bootstrap,
and its standalone application connects through Aegis's service-free public
Unix client with a concrete same-session grant provisioner for exact
conversation or scheduled-composition actions. The deployment still supplies
the administrator credential mapping, authority epoch, grant lifetime,
resource, and opaque subject. Agora now also owns an ordered durable create, clean-open, and
exact endpoint-residue-recovery factory. Its shared owner loop now accepts an
idempotent shutdown wakeup and returns the exact drain/cleanup receipt. Its
bounded standalone configuration and POSIX signal wrapper reuse that owner;
the real standalone journey and Workbench-supervised exact process now qualify
create, public-client observation, clean drain, and durable reopen. Workbench
receives only the executable, opaque config path, public socket, and explicit
composition transport; it does not interpret authority or bootstrap facts. A single Workbench process and lock
closure now own all three peers and drain them in reverse order under an
explicitly test-only Agora authority composition. While all three owners are
live, that qualifier also runs the reusable headless aggregation API used by
the standalone CLI and retains a current observation for every peer. It then
feeds a second real Agora client into the reusable component and renders its
room and message projections through Workbench's shell boundary. A second
scenario returns Agora's public scheduled-composition client from the same host
registry, publishes, observes the retained result, and drains cleanly; it does
not upgrade the test authority into production provenance. Chronos standalone and Agora
daemon select the compatible checkpoint/commit subset under different source
identities for `keystone-storage`; the qualifier derives a scoped local Cargo patch to prove
the all-three lifecycle without embedding a machine path. The selected
production topology instead supervises Agora's exact standalone process, so it
does not merge the Chronos and Agora Cargo source graphs or require an
architecture-driven Keystone convergence.

The registry also exposes explicit deployment-owned factory seams for Moira,
Daimon, Hermes, Aegis, and Grounding. Each returns its concrete typed client and
contributes only an operational shutdown callback; Workbench does not construct
their stores, provider policy, retention policy, grants, trusted time, or MCP
semantics. These seams are not standalone profile factories and do not close the
host matrix until real owner lifecycles are qualified in both product modes.

The supported mixed topology is now qualified independently: Workbench can own
Chronos and Vellum while attaching Agora, and can present and observe all three
in one interactive or headless invocation. Together with the all-three owner
qualification this proves the composition and lifecycle seams separately; it
does not substitute test authority or attachment for the still-open production
Agora owner factory. The production-shaped alternative supervises Agora's exact
standalone process rather than linking its engine graph and is qualified with a
real Workbench-hosted Aegis owner in both headless and interactive modes.

### 4. Qualify headless parity

The headless observer is now an application-library API rather than CLI-private
code. Deployments can supply the same typed Agora profile, Chronos workstreams,
and Vellum endpoint used by the standalone parser and receive the same bounded
peer observations without constructing TUI state. The API independently checks
every requested capability against the peer-owned headless manifests before
opening any connection, so a deployment cannot bypass negotiation by omitting
the CLI parser.

- Run the same hosted topology without Ratatui, focus, selection, or cursor state.
- Provide bounded per-peer command and snapshot routing plus aggregate health;
  keep semantic results typed by the owning peer.
- Exercise restart, one-peer failure, lost replies, cancellation, and shutdown.

Exit check: interactive and headless journeys produce the same peer-owned
receipts/refusals for equivalent operations and never infer aggregate success
from presentation or process state.

Status: attached read parity is qualified for Agora, Chronos, and Vellum,
including one unavailable Chronos workstream alongside healthy peers. Hosted
Chronos/Vellum read parity and clean reverse shutdown are qualified in both
product modes; their headless aggregate publishes lifecycle receipts before
claiming success. An injected, deployment-owned Agora factory is also qualified
against the real daemon in the same process as the Chronos and Vellum owners,
including reusable interactive rendering, the shared headless observer, and
clean reverse shutdown. The same lifecycle seam also qualifies scheduled
publish/observe through Agora's public composition client under test-only
authority. Durable Agora restart now passes through the exact standalone
process in both standalone and Workbench-supervised journeys. An induced
non-clean Agora receipt now proves aggregate refusal, continued reverse drain
of Aegis, and durable reopen. A separate shutdown-edge process cut proves
abrupt child death, visible endpoint residue, safe replacement readiness, exact
incarnation rotation, and retained-room recovery. A bounded missing-receipt cut
also proves `outcome_unknown`, continued Aegis drain, and the same explicit
residue recovery. Grant provisioning stays owner-side and outside Workbench.

Chronos's selected guarded-write profile is also qualified through the hosted
Workbench path using the peer-owned public Hub: task creation, receipted
cancellation, scheduling enablement, global/task pause, capacity, and
not-before controls survive clean drain and exact independent service reopen.
This does not claim mutations beyond that profile.

### 5. Extend peer coverage

Moira now has an independent reusable TUI, standalone interactive/headless CLI,
peer-owned `sagas.inspect.v1` capability, content-free aggregate census, and
Workbench attach composition through the same read-only client. Its
read-only component exposes bounded saga, reconciliation, effect-evidence, and
operator-receipt projections through the public client. The real-daemon journey
is qualified interactively both standalone and through Workbench, while the
headless path preserves the content-free census. Operator reconciliation remains
deliberately absent until its confirmation, command-scoped authority, revision,
durable receipt, and uncertain delivery contracts can be preserved end to end.
Native Workbench hosting also remains open.

Daimon now likewise has an independent reusable TUI, standalone CLI,
`agents.presence.v1` headless observation, and Workbench attach composition. Its
default surface remains deliberately redacted to age and stale/current
classification. An explicit opaque profile selector now reuses
`agents.session.inspect.v1` in standalone and Workbench modes for stable phase,
provider-profile digest, capability and bounded event evidence without exposing
a session catalogue. It cannot dispatch, prompt, decide permission, advance,
cancel, close, resume or reattach a provider session. Guarded operations remain
later slices.

Hermes now has an independent reusable ledger TUI, standalone CLI,
`ledger.observe.v1` status/usage/signals observation, and Workbench attachment.
An exact opaque profile selector now carries content-free stream health through
the same standalone and Workbench component without inventing a global stream
catalogue or echoing the identifier. A second exact opaque selector now carries
one consumer's content-free cursor position, backlog, and retention-gap status
through the read-only facade in both products; the presentation never acquires
acknowledgement authority. A separate `ledger.events.disclose.v1` command now
reuses Hermes's `DisclosureClient` and exact stream/start/limit guard in both
standalone and Workbench, bounded to 16 events plus the negotiated frame and
reporting retention/publication gaps explicitly. Append and compaction remain
guarded later operations.

Hermes now also supplies the shutdown-aware accept loop and explicit observer
and exact-stream operator daemon profiles. Workbench owns that sovereign process
through explicit profile inputs, waits for its owner socket, attaches the
ordinary public client, and records bounded signal shutdown. This avoids
combining Hermes and Chronos engine graphs or masking their distinct
`keystone-storage` sources with a product Cargo patch. Headless create/reopen
lifecycle is qualified. The interactive Workbench path now renders the same
public-client observation, drains the owner on shell exit, removes the socket,
and proves a subsequent headless reopen. Append remains producer-specific.

Hermes acknowledgement now has one deliberately narrow authority-bearing path.
The standalone and Workbench commands reuse the same owner guard: exact
confirmation precedes traffic, the current cursor is observed and compared on
the same connection, and one acknowledgement is attempted. A stale cursor
prevents mutation and a post-send transport failure remains outcome-unknown
without retry. The reusable component retains no consumer authority; hosted
consumer authority requires the explicit exact-stream operator profile. Generic
append remains open.

Hermes compaction follows the same pattern through a distinct owner guard. The
standalone and Workbench commands observe the retained frontier and published
edge, compare both with exact operator expectations, and attempt one confirmed
compaction on the same connection. Stale state prevents mutation and uncertain
delivery is never retried. `OperatorClient` remains outside the reusable
component; any hosted operator authority is an explicit exact-stream Hermes
process input. Generic append remains open because its domain envelope belongs
to a producer-specific integration.

Aegis now has an independent read-only authority TUI, standalone CLI,
`authority.observe.v1` bounded status/grant-summary/decision-summary observation,
and Workbench attachment qualified against the real durable daemon in both
interactive and headless modes. The shared component now navigates forward and
backward across bounded redacted grant/decision pages while headless observation
retains one bounded page plus explicit continuation evidence. Standalone and
Workbench also reuse Aegis's peer-owned `authority.revoke-epoch.v1` guard: exact
literal confirmation, same-connection current-epoch observation, one mutation
attempt, and preserved outcome uncertainty are qualified against the real
daemon. The same owner guard now supplies a bounded
`authority.attenuate-authorize-once.v1` journey: issue, attenuation, and one
authorization share a connection; no grant handle escapes; and every
post-send failure remains uncertain without retry. Aegis's bounded
`aegis.admin-session.v1` application protocol now supplies reusable named issue,
attenuation, authorization, listing and close in both standalone and Workbench
shells. Both shells call the same owner parser/manager over the public client;
Workbench owns no grant handles, confirmation grammar or retry policy. Two
separate Workbench command processes against the selected owner daemon qualify
disconnect abandonment, an empty fresh name set, and deliberate rebuild under
a new operation ID. Durable grant replay and lost-reply reconciliation are
nonclaims.

Aegis now also supplies a standalone shutdown-aware owner daemon. Its explicit
deployment inputs select create/open recovery, authority namespace, exact kernel
peer mapping, principal, session namespace and administrator/principal role.
Workbench supervises that sovereign process, waits for its socket, connects the
ordinary public client and records signal-drain receipts. Headless create/open
and interactive reopen are qualified without linking Aegis engine, service,
storage or authentication implementations into Workbench. Long-lived grant
management remains an Aegis-owned application contract, not hosting authority.

The combined Moira, Daimon, Hermes, and Aegis real-daemon attachment gate is in
place for one headless Workbench invocation and now also observes Grounding
through Dendrite's typed endpoint. The current qualification passes against all
five real services, including Hermes's configured-stream and Moira's fenced-saga
census. Moira's selected evidence and operator receipt are additionally
qualified in its isolated standalone/Workbench interactive journeys. This
remains local working-tree evidence;
it does not imply native hosting, immutable source closure, or shared mutation
authority.

Grounding now has a typed authenticated Dendrite presentation endpoint,
independent reusable TUI and standalone CLI, `grounding.observe.v1` provenance
and freshness observation, and Workbench attachment qualified against a real
Dendrite fixture in interactive and headless modes. MCP remains an agent-facing
surface and is not Workbench's internal protocol. Selected content navigation
remains a later guarded slice. Native Workbench hosting is intentionally not
applicable: Dendrite retains its independently deployed IDE/agent bridge
lifecycle.

## Qualification matrix

Every claimed applicable peer cell must pass:

| Owner | Interactive | Headless |
| --- | --- | --- |
| Peer standalone CLI | Reused peer TUI | Peer client commands without UI state |
| Workbench attach | Reused peer TUI over remote client | Same remote client graph |
| Workbench host, when applicable | Reused peer TUI over hosted client | Same hosted service/client graph |

The matrix includes authorization denial, stale revision, service unavailable,
outcome unknown, recovery, and orderly/partial shutdown cases.

Current status: the standalone and Workbench-attach read cells pass for Agora,
Chronos, and Vellum with explicit capability IDs and versioned configuration or
service failure envelopes. All three standalone paths have real-service gates.
The Workbench-host row passes for Chronos, Vellum, Hermes, Aegis and Agora in
both modes, including durable create/open and clean owner-process drain. Agora's
production-shaped profile supervises its exact standalone process from an
opaque owner config and a real Aegis owner. The older in-process factory remains
test-only authority evidence and is not promoted to production provenance.

Presentation and invocation parity are therefore closed for the current
read-only peer surfaces: each peer supplies a reusable component, a separate
standalone CLI, an explicit headless path, a peer-owned capability manifest,
and every applicable Workbench attach or host composition. Production
activation is a separate deployment claim, gated by immutable per-owner source
and artifact evidence. Workbench must not force an unselected in-process graph
or absorb owner policy merely to make the matrix appear uniform.
