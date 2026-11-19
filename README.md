# Pantheon

Pantheon gathers the suite's independently governed peers beneath one
application roof without absorbing their semantics or authority. Its reusable
terminal presentation host and sibling `pantheon-cli` package divide ownership
cleanly: the CLI owns standalone argument parsing, terminal lifecycle, and process exit;
the root package owns navigation, layout, focus, and global input routing. Each
attached peer renders only inside its assigned rectangle and retains its own
input and service authority.

The active terminal-experience architecture, precedent study, capability map,
and delivery sequence live in
[`docs/tui-experience.md`](docs/tui-experience.md). The broader product model,
component use cases, and aggregate journey acceptance catalogue live in
[`docs/use-cases-and-user-journeys.md`](docs/use-cases-and-user-journeys.md).
The active journey delivery ledger is
[`docs/journey-execution-plan.md`](docs/journey-execution-plan.md); architectural
decisions live under [`docs/decisions/`](docs/decisions/), and initial
declarative work-mode definitions live under
[`docs/work-modes/`](docs/work-modes/). Hestia owns model-artifact admission and
inference-runtime placement; Pantheon now composes both owner-published Hestia
surfaces without gaining download, lifecycle, qualification, or execution
authority. Oculus owns bounded observability inquiry; its Pantheon composition
remains planned rather than claimed.
The interactive shell now
provides a searchable `Ctrl-P` Emacs-style minibuffer, generated `F1` help, and a
bounded `Ctrl-G` activity/receipt timeline. `[` and `]` move across the adaptive
surface tab rail. On enhanced-keyboard terminals, stable semantic shortcuts use `Ctrl-0` for Overview,
`Ctrl-1` through `Ctrl-9` for Chronos through Operations, and `Ctrl-,` for
Configuration; equivalent `Alt` numbers and mnemonic shortcuts
(`Alt-H/C/V/A/M/D/G/E/L/O` and `Alt-,`) remain available. Clicking a tab or
wheeling over the rail also navigates. These bindings open every first-class
surface, including the redacted Configuration pane; unavailable guarded work remains
discoverable with an exact reason instead of disappearing. The palette also
exposes a redacted session-configuration sheet, whole-suite refresh, Chronos
work forms, peer detail navigation, and Moira's selected-saga keep-uncertain,
fail-terminal, cancel, and compensate reconciliation sheets plus Daimon's exact advance/cancel/close sheets without
exporting peer authority into the shell. Agora also exposes a conversation-native
room navigator, an exact revision-bound room-creation sheet, and a bounded
multiline message composer with canonical multi-recipient addressing and exact
delivery recovery. A content-free
instrumentation sheet reports bounded local dispatch, frame render, peer-read,
guarded-control, in-flight, outcome, and full settlement latency metrics with
visible budget violations. Its bounded trace ledger groups multi-subsystem work
under one content-free correlation and shows shell dispatch, peer pending,
receipt, and terminal settlement stages without claiming unobservable peer
internals. It never records prompts, documents, credentials, grant handles, or
provider locators. An Operations surface also exposes the live local-host
lifecycle, hosted-peer census, guarded reverse-order drain, and retained exact
per-peer shutdown receipts.

On the Chronos surface, `Shift-N` (or `M-x Create task with local agent`)
starts the cross-peer work-item journey. Pantheon retains the bounded task text
entered in the peer form, waits for an applied Chronos receipt, and then opens a
prefilled, guarded prompt for the selected local Daimon session. The user still
reviews, arms, and confirms the provider effect. `M-x Open the local-agent
conversation` returns to its event timeline; `M-x Return to the Kanban board`
returns to Chronos and queues a fresh owner projection. This revision does not
pretend to open a new Daimon session or write comments into Chronos: those need
owner-published dynamic task-binding/session-open and ticket-comment contracts.

`make run` is the operational local-development path: it hosts writable
Chronos plus editable/saveable Vellum seeded with this repository's README,
with runtime file admission enabled. Override `PANTHEON_RUN_ROOT` or
`PANTHEON_RUN_VELLUM_ROOT` (an initial regular file) to isolate its state or
seed a different workspace.
Ctrl-O and Chronos `n` are therefore functional in the default run target, not
merely advertised by an unconfigured surface.

To inspect the existing journey-development TUI from another shell, run
`make attach`. It attaches that terminal to the exact `pantheon-j1-next` tmux
session and does not start, replace, or rediscover any Pantheon or peer owner.
Select another explicitly known presentation session with
`make attach PANTHEON_TMUX_SESSION=<name>`. From inside tmux the same target
switches the current client, avoiding a nested session.

With an exact Hermes stream and consumer selected, the Hermes surface can also
acknowledge an exact operator-selected range from the current cursor through
the observed published edge via Hermes's owner guard. The sheet requires
arm/confirm, makes one attempt on a fresh same-connection re-observation,
fences uncertain delivery, and keeps the instrumented operation open until the
refreshed cursor projection settles. With an exact stream selected, the surface
can disclose an operator-selected 1–16 event batch through Hermes's owner guard
after explicit sensitive-content confirmation. The worker immediately reduces
each response to event metadata and at most 256 payload preview bytes; content
never enters activity receipts or telemetry.
Pantheon's trace spans confirmed dispatch, the Hermes read boundary, the peer
receipt, and terminal settlement, including refusal and transport failure.

The binary attaches a Chronos Hub and one Vellum Unix service socket. Add
`--chronos-workstream NAME DIRECTORY` for each extra Chronos service. With
Chronos focused, **w** switches workstreams and **/** searches observed tasks. It does not orchestrate agents, merge engine state or infer
task completion from edits. The host preserves Vellum's dirty/indeterminate exit
laws and never retries an uncertain effect. Chronos writes remain disabled
unless explicitly enabled; Vellum authority remains entirely service-owned.

The versioned workspace shell also names Overview, Chronos, Vellum, Agora,
Moira, Daimon, Tellus, Aegis, Hermes, local-host Operations, and
Configuration surfaces. Chronos and Vellum receive independent full-frame
presentations; legacy profiles may still select the side-by-side `workspace`
surface. Overview reports
each configured peer channel as configured-but-unobserved, observing, current
with evidence age, or degraded; live control state adds busy and
outcome-uncertain markers without turning configuration into a readiness claim.
Overview and the independent Chronos and Vellum surfaces are live in the standalone binary. A workspace profile may attach Agora through
the selected private Unix client by supplying its socket and participant claim;
the daemon authenticates the connected OS peer and remains the authority owner.
A trusted host can instead supply an independently constructed `agora-tui`
component through `pantheon_cli::run_hosted_terminal_with_agora`. Moira,
Daimon, Hermes, Aegis, and Tellus profiles attach their peer-owned read-only
components. Tellus uses Dendrite's typed presentation endpoint rather than
MCP tool envelopes or text. `[` and `]` switch
surfaces; Tab switches focus only inside Workspace. See
[the gap ledger](WORKSPACE_GAPS.md) and [delivery roadmap](ROADMAP.md).

From a workspace-profile session, `Ctrl-P` → **Preflight and restart workspace
profile** opens a bounded local path editor. Enter validates the candidate with
the same profile parser used at startup and renders the exact typed diff and
minimum restart boundary. **c**, then Enter, requests one process replacement
only after guarded work is settled, Vellum persistence is clean, peer clients
disconnect, and Pantheon-owned topology drains cleanly. The replacement keeps
the original interactive flags and changes only the preflighted profile path;
there is no live-apply claim or implicit retry. The handoff binds the exact
preflighted profile bytes with SHA-256: confirmation revalidates the file, and
the replacement process refuses startup if the digest no longer matches.

```sh
cargo run -p pantheon-cli --bin pantheon --locked --offline -- interactive \
  /private/tmp/chronos-service-directory \
  --vellum-service /private/tmp/vellum.sock
```

To load a bounded workspace profile instead, copy
`config/workspace.example.json`, adjust its relative service paths, and run:

```sh
cargo run -p pantheon-cli --bin pantheon --locked --offline -- interactive \
  --workspace-profile config/workspace.json
```

Validate profile structure and path resolution without contacting a service:

```sh
cargo run -p pantheon-cli --bin pantheon --locked --offline -- interactive \
  --workspace-profile config/workspace.json --check-profile
```

Profiles contain connection configuration, presentation selection and explicit
cross-service scope mappings. They cannot enable Chronos writes or carry grant
handles. Continue to pass `--chronos-allow-writes` at launch when intended.
Agora, Chronos, Vellum, Moira, Daimon, Hermes and Aegis profiles may select `"deployment":"hosted"`. Pantheon then
owns their real local service lifecycles while the same peer clients and TUI
components are used. Agora, Daimon, Hermes and Aegis remain sovereign supervised owner
processes rather than linked semantic engines. Hosted Vellum requires a bounded non-empty `roots` list;
attached profiles must not include roots. Edit and save authority remain
launch-only through `--vellum-allow-edit` and `--vellum-allow-save`.
Each configured peer may also carry a bounded `requires` list. Pantheon checks
those exact identifiers against its own resolved route catalog for the selected
interactive/headless mode, attach/host topology, configured selectors and
launch-only feature flags before connecting. It does not accept a capability
merely because a separate command exists somewhere in the peer TUI package.
`--check-profile` prints the resolved observation, operator, control and
management-plane capabilities without contacting a peer. A match describes
composition support only; the service still owns authorization and may refuse
the request. Hyphens are valid in the bounded capability identifier grammar.
The Agora participant value is an identity claim checked against the daemon's
authenticated peer context; it is neither a credential nor authority.
Unknown fields, versions, duplicate surfaces, control characters, excessive
workstreams/bindings and files larger than 64 KiB are refused.

A Daimon profile pairs `agent_socket` with either the backward-compatible exact
32-hex-digit `agent_session` or a bounded, unique `agent_sessions` list of up to
16 exact sessions. The list is an explicit local workset, not owner-side
discovery or a fleet claim. Pantheon reuses Daimon's read-only client to present
stable session views and bounded event pages. Left/Right moves between sessions,
Space sparsely marks sessions, `r` refreshes the focused session, and `R`
refreshes the set; with no marks, actions target only the focused session. The
**f** key cycles all, conversation, action, decision, and usage event evidence.
The footer reports exact configured/current/unavailable/marked cardinality plus
visible/observed event counts and summed input/output usage from the bounded
owner-published windows; these are projection facts, not provider cost or
execution-policy inference. The
advance, cancel, and close sheets execute a sequential, non-atomic guarded run:
each child receives a fresh owner connection and operation identity, retains
its returned durable session projection, and remains separately Applied,
Refused, Failed, OutcomeUnknown, or NotSent. An uncertain or failed child stops
later sends. When a presence socket is also selected, the interactive Daimon
surface renders presence beside the focused workset rather than hiding it.
Profile selectors confer no open, prompt, permission, reattach, cancel, or close
authority; every mutation remains owner-guarded.

A hosted Daimon profile requires that exact agent selection plus an owner
`host` containing only the `daimon-local-agent` executable and its opaque
configuration path. The owner configuration retains provider commands,
inherited credential names, Aegis/Chronos endpoints, checkpoint directory and
limits. Pantheon invokes Daimon's `--supervised` lifecycle, waits for the
configured public agent socket, uses the same public client as attach mode, and
requests a bounded owner-defined drain on exit. A presence socket remains
optional because the production agent application does not claim to serve the
separate legacy presence protocol.

A hosted Moira profile supplies only the `moira-saga-local` executable and its
opaque owner configuration. That Moira document retains the database, kernel
peer mapping, all bounds and the selected operator-authority policy. Pantheon
invokes the exact supervised lifecycle, waits for the private saga socket and
attaches through `moira-client::unix::SagaUnixPort`; it cannot turn ordinary
read/mutate access into reconciliation authority.

The separate `moira-reconciliation-view`, `moira-reconciliation-confirm` and
`moira-reconciliation-recover` commands reuse Moira's owner-side guard. They
accept only an exact server-prepared challenge, submit confirmation once without
retrying an unknown outcome, and resolve uncertainty only from the exact durable
operator receipt. Pantheon never parses the Aegis portion of the owner config.
Moira-TUI's newer agent and deliberation capabilities are not silently inherited
by this saga route. Governed-agent participation is deliberately composable:
pair an attached `participation_socket` with one exact 32-hex-digit
`participation` in the Moira workspace-profile object. Pantheon then uses
Moira's independent public client, advertises
`agent.participation.inspect.v1`, emits the selected content-free observation in
headless mode and presents the owner record beside sagas in the Moira tab. The
saga host does not implicitly serve this separate protocol, and Pantheon neither
copies the record nor derives authority from its persona or phase.
On the Moira saga list, Space marks up to 16 sagas; with no marks, the focused
saga remains the one-item default. The keep-uncertain, fail-terminal, cancel,
and compensate palette actions open one parent workflow but never claim an atomic batch. Each path re-inspects every child
immediately before submission, retains an exact durable receipt or refusal per
child, stops on an unknown outcome, and leaves later children visibly not sent
until exact recovery and an explicit re-arm. Time-bearing proposals require an
explicit bounded Moira logical-time value—never inferred wall time—and show it
beside each selected saga before owner inspection or arming. The real batch gate
proves every disposition class, including effect-bound failed/successful
evidence admission and retry, then independently reopens the owner to verify
the durable phases and exact receipts.

Use `--chronos-profile PATH` in place of the initial service directory to
reopen a workspace saved by `chronos-tui --save-profile`. Relative service
paths resolve from the profile directory. Peer flags use the exact names in
the profile, and write access remains an explicit launch option.

Add `--chronos-allow-writes` only when the Chronos service was composed for
writes. `--chronos-watch` enables feed recovery on services started with
`--atomic-state --observer-history`. `--chronos-scheduling` exposes scheduling
controls. `--chronos-planning PATH` loads the same read-only Gantt source as
the standalone Chronos Hub. Press **7** to view the plan.
`--chronos-cancel-receipts` selects V3 cancellation on a writable
atomic service. Add `--vellum-credential PATH` for Vellum's optional local credential
handshake. In hosted mode the same private record configures both the daemon
verifier and client claim provider without placing credential material in the
profile or lifecycle receipt. Tab changes focus, Ctrl-O opens Vellum's bounded
file-admission prompt, and Ctrl-Q requests
whole-workbench exit.
Hosted sessions must opt into runtime file admission with
`--vellum-allow-add-file` or `"allow_add_file": true`; Pantheon maps that
selection to Vellum's native `AdmitFile` ceiling and otherwise preserves the
owner's refusal. Attached sessions inherit the independently launched Vellum
daemon's action ceiling.
The Chronos new-task form accepts one task directly or a bounded batch of up to
128 unique IDs by adding rows with Tab. Enter sends the exact set as one
owner-atomic `SubmitGraph`; Pantheon does not issue one mutation per task.
For cancellation, Space marks tasks on the current bounded owner page and `c`
opens one confirmation for that set (or the focused task when no marks exist).
Every child uses an independent V3 invocation receipt and a freshly observed
owner revision. An unresolved receipt stops the run and leaves later children
not sent; Pantheon traces the complete run as `chronos.task.cancel-batch`.
The same marked/current workset drives task pause and not-before. Pantheon
reuses Chronos's V5 client, reobserves exact eligibility before each child,
preserves each task's unrelated field, exposes `sA/sR/s?/sN` dispositions, and
traces the run as `chronos.schedule.eligibility-batch`. Both actions are
discoverable from `Ctrl-P`; V5 uncertainty stops later sends and is never
restyled as atomic success.
The exact interactive Chronos route advertises `tasks.mutate.v1` only with
`--chronos-allow-writes`, `scheduling.observe.v5` with
`--chronos-scheduling`, `scheduling.mutate.v5` only when both selections are
present, and `tasks.cancel-receipt.v3` only for writable receipted cancellation.
Watch, planning, and selected Moira/Daimon attempt projections are likewise
conditional route capabilities. Headless Chronos remains observation-only.
Use the explicit `headless` command for a bounded, versioned JSON observation of every configured
Agora, Chronos, Vellum, Moira, Daimon, Hermes, Aegis, and Tellus peer. Each row names its peer-owned capability and
truthful status. Healthy rows remain visible when another peer is unavailable,
but the command refuses aggregate success. `--headless` and `--snapshot` remain
compatibility aliases for invocations without an explicit command.
The same report includes a `management` object for the first supported
deployment scope, `single_user_local_machine`. It classifies attachment versus
hosting, readiness at observation time, revision when the peer supplies one,
certainty, fencing, the peer-owned read-only authority path, and the current
reconfiguration boundary. Hosted rows are reconciled with exact reverse-drain
receipts before output; attached rows remain `external`. Attachment alone is
never reported as readiness, and a missing peer revision remains `null`.
Every headless report also carries one bounded `operation_id` in both the
top-level envelope and management snapshot, tying all peer observations and
hosted drain receipts to the same local operation. Supply it with
`--operation-id ID` for an external audit/recovery workflow, or let Pantheon
generate a process-and-time-scoped local identifier. The identifier is
correlation only: it grants no authority and is never forwarded as a peer
request identity or idempotency key.

Run `pantheon reconfigure-plan CURRENT_PROFILE NEXT_PROFILE` to compare two
fully validated local profiles without opening peer connections. The versioned
JSON plan classifies workspace-shell and per-peer changes and reports the
minimum safe action. The current boundary is explicit: shell-only changes need
a shell restart; any peer addition, removal, endpoint, selector, deployment,
authority input, or requirement change needs a full local-topology restart.
Live apply is reported as unsupported. An optional `--operation-id` connects
the plan to the subsequent headless drain/restart evidence.

Embedding deployments also have a bounded runtime `NativeHost::restart_peer`
primitive. It targets one exact hosted peer, returns the replacement's concrete
client plus a typed stop/start receipt, and invokes the replacement factory only
after a confirmed clean stop. Refused or outcome-unknown stops fence further
reconfiguration by moving the registry to `Degraded`; final shutdown still
reports that unresolved peer and drains every unaffected sibling. A clean stop
followed by replacement refusal likewise leaves a degraded but drainable
topology. This is the execution substrate for later CLI plan application, not
permission to reinterpret any peer's state or authority.
`NativeHost::restart_topology` wires a validated full-topology plan to that
substrate: it drains the old host first, assembles and seals the replacement as
one unit, returns the caller's concrete typed client bundle, and rolls back a
partial replacement. Its correlated receipt records the old drain and exact
replacement peer set. Standalone CLI plan application remains intentionally
separate until it can retain a long-running replacement instead of immediately
draining a one-shot headless invocation.
While an embedding host is live, `NativeHost::status` returns the registry
lifecycle, exact hosted-peer set, whether reconfiguration is currently allowed,
and retained restart receipts. A degraded topology therefore remains
inspectable after a refused replacement or uncertain stop instead of losing the
evidence that fenced further changes. This is an in-process management API; no
remotely reachable or ambient-authority control endpoint is claimed.

`pantheon manage --workspace-profile PATH` exposes that boundary as a
long-running, bounded local stdio session. It emits one-line versioned JSON and
accepts only `status`, `observe [OPERATION_ID]`,
`reconfigure OPERATION_ID NEXT_PROFILE`, and `drain`. Observations use
the same public clients while hosted peers remain live; `drain` and EOF both
perform the ordinary reverse shutdown and emit exact receipts. Commands are
bounded to 4 KiB, unknown commands are refused, and the session opens no socket,
does not interpret peer mutations, and gains no authority beyond the supplied
profile. This is suitable for a single-user parent supervisor; service-manager
and authenticated IPC profiles remain separate work.
The protocol is explicitly one-request-in-flight: a parent waits for the
corresponding event (and, after reconfiguration, the replacement's `ready`
event) before sending another command.

`reconfigure` loads and fully capability-checks the replacement profile before
touching the current topology. No-change requests return in place. A real
change drains the old topology, emits its exact correlated receipt, and then
replaces the Pantheon process with the same executable and the validated next
profile while retaining the stdio channel. The replacement emits a new `ready`
event under the requested operation identifier. A refused or uncertain drain
fences the handoff, and an exec or replacement-start failure remains explicit;
this is a bounded process restart, not a claim of live mutation.

Run `pantheon controls` for the machine-readable local guarded-control catalog.
Each entry maps one qualified capability to its concrete CLI command and marks
it as a separate control-plane route with peer-owned authority and
route-specific confirmation. These entries are deliberately absent from the
aggregate workspace capability route; discovering a command does not grant
permission to execute it.

A Vellum-owned integration host can also call `pantheon_cli::run_hosted_terminal`
with a Chronos `Hub` and a factory for the Vellum service and handoff handler.
Editor commands and handoff requests share one service owner. The host resolves
the document association and authorizes capture, publication, and review.
The standalone socket CLI starts with handoff controls disabled.

The Pantheon executable can natively host Chronos and Vellum together in
interactive or headless mode. Services bind before clients are constructed;
startup refusal rolls back earlier owners, and shutdown drains in reverse order
with explicit clean, refused, or outcome-unknown receipts. It can also supervise
Agora's exact `agora-private-local` process from an opaque owner config, wait for
its public socket, select the routed public client explicitly, request Agora's
stdin drain, and validate the bounded cleanup receipt. Pantheon never parses
the config's Aegis, grant, clock, bootstrap, or durable-storage facts.

The workspace-wide standalone/Pantheon pattern, composition taxonomy, current
parity matrix and Axon/Engram prior-art routing live in
[`docs/composition/`](docs/composition/README.md). Peer roadmaps continue to own
their behavior; this repository owns product composition and its evidence. The
closed machine-readable parity matrix is [`composition-status.json`](composition-status.json).

The separate `agora-create-room` command reuses Agora's owner-client
`rooms.create.v2` guard rather than routing a generic Pantheon mutation. It
accepts the explicit conversation or composition transport, exact room facts,
expected revision and literal confirmation; an inexact confirmation is refused
before connection and an uncertain outcome is never retried by the shell.
The interactive `agora.message.compose` route similarly reuses the peer-owned
`messages.speak.v2` client. It is intentionally conversation-native rather
than a generic mutation command: open a room, compose a multiline body, add an
optional complete recipient set, review, arm, and confirm. Pantheon records
only timing and the owner receipt class; message and participant content never
enter its telemetry.

Local coordinated source layout:

```text
gantbd/
  chronos/
  chronos-tui/
  daimon/
  daimon-tui/
  hermes/
  hermes-tui/
  aegis/
  aegis-tui/
  dendrite/
  tellus/
  keystone/
  moira/
  moira-tui/
  vellum/
  vellum-tui/
  pantheon/
```

Run `make check`. Nothing is published to crates.io; this repository
distributes source only.
The machine source receipt records the clean committed local closure. Remote
CI and release qualification remain deferred until those exact inputs are
remotely reachable.

The source receipt now pins the committed Aegis, Agora, Chronos, Daimon, Tellus, Hermes, Moira, and Vellum TUI
boundaries used by this composition. Their reusable default presentations keep
standalone CLI/application concerns outside Pantheon's dependency graph.
It also pins Keystone as an explicit substrate source input because selected
owner crates resolve its local mechanisms; this records source closure without
making Keystone a product, presentation, or semantic integration layer.
Remote reachability remains unqualified. Active working trees are never copied
or inferred as release evidence.

Run `python3 scripts/qualify-source.py WORKBENCH_COMMIT --live` to run the
locked Rust suite and every deduplicated real-service gate named by the
candidate's `composition-status.json`, all from temporary archives of the
candidate commits. The command reads the peer
pins from the source receipts and accepts explicit full candidate commit IDs
for peer owners while evaluating an updated set. Before entering offline mode,
it seeds a private Cargo Git cache from the exact archived Keystone object
under the declared GitHub identity; it never relies on ambient Git checkout
state or remote reachability. It emits the tested revisions after success and
leaves the sibling checkouts untouched.

For attempt inspection, pass `--chronos-moira WORKSTREAM SOCKET SCOPE`
and optionally `--chronos-daimon WORKSTREAM SOCKET`. Use `Chronos` for the
primary workstream or the exact name supplied with `--chronos-workstream`.
With Chronos focused, press **5**, select an attempt, and press **x**.
Moira authorizes its receipt disclosure. Daimon authenticates and authorizes
a separate inspection when Moira supplies a recorded execution ID.

For independent product-manager alignment, pass
`--chronos-alignment WORKSTREAM DIRECTORY SCOPE`, where `DIRECTORY` is the
private Chronos service directory containing `chronos.alignment.sock`. The
binding must name one exact configured workstream and scope. Interactive mode
adds that workstream's read-only **8 Alignment** tab; press **r** to refresh.
Headless mode emits the same bounded, content-free observation. Neither route
can apply a proposal, accept work, claim tasks, reprioritize the board, or sign
off for a human. The product manager recommends and challenges through the
Chronos-owned record; accountable owners and explicit human gates retain those
decisions.

Run `sh tests/live/dual_service.sh` for local composition qualification.
It first builds and attaches temporary Agora, Chronos, and Vellum services,
reopens a saved profile with relative service paths, and uses an isolated tmux
session to check workstream switching, offline recovery, focus routing, all
three peer panes, and exit. It then restarts Chronos and Vellum under Pantheon
ownership while attaching Agora, qualifies the mixed topology's headless
snapshot and interactive panes, and proves reverse shutdown removes every
Pantheon-owned data and control endpoint.

Run `sh tests/live/chronos_mutation.sh` for the selected Chronos mutation
profile through the real Pantheon-hosted peer component. The isolated journey
creates three tasks in one atomic graph, enables scheduling, changes capacity
and not-before across all three marked tasks, pauses all three tasks and the
scheduler, performs three independently receipted cancellations, drains
Pantheon, then
reopens the state with an independent `chronos-service` and verifies the exact
durable result. This qualifies that profile locally; it does not invent mutation
authority or grant wider authority. Pantheon's route catalog names the selected
control boundaries as `tasks.mutate.v1`, `scheduling.mutate.v5`, and
`tasks.cancel-receipt.v3` without replacing Chronos's protocol or authorization.

Run `sh tests/live/vellum_mutation.sh` for ordinary witnessed Vellum mutation
parity. Pantheon hosts the real owner with explicit edit/save ceilings, drives
the same `ServiceEditor` used by the standalone peer, verifies exact persisted
bytes and clean drain, then repeats without save authority and proves the
service refusal leaves source bytes unchanged. The standalone Vellum gate owns
the complementary lost-reply proof: `Indeterminate` resolves only by replaying
the exact retained save request.

Run `sh tests/live/vellum_durable_save.sh` for the selected restart-stable Save
profile. One authenticated real Vellum daemon receives the caller-persisted
operation/document/revision through Vellum-TUI's public-client guard; the
standalone command executes it and the Pantheon command performs only exact
historical lookup. Authentication, the Persist ceiling, filesystem work and
receipt truth remain Vellum-owned. The gate does not expose Save As submission,
receipt retirement, arbitrary destinations or other durable intents.

Run `sh tests/live/vellum_batch.sh` for the integrated N-document workflow.
Pantheon edits and sparsely selects three service-owned documents, re-observes
their exact revisions, and invokes Vellum-TUI's independently receipted durable
save component. The authenticated owner persists all three exact byte streams;
Pantheon admits clean state only for children with durable receipts. The route
is advertised and the workbench action is offered only when a stable Vellum
credential is configured. The workflow makes no atomic-batch claim: unknown
delivery stops later children as not sent until exact lookup and explicit
re-arm.

Run `sh tests/live/vellum_atomic_replace.sh` for the owner-atomic N-document
edit path. The journey marks three documents, enters one bounded literal
query/replacement, previews four matches against three exact revisions, and
publishes one `WorkspaceEdit` generation. It then inspects the correlated
`owner_atomic_receipt` trace before invoking the separate per-document durable
save workflow and verifying all exact filesystem bytes. The composer fences an
unknown reply and can recover only by replaying the retained request; it never
restyles in-memory atomicity as cross-file durability.

The selected local release profile is documented in
[`docs/local-release.md`](docs/local-release.md). It reconstructs an immutable
source closure, builds one unsigned `aarch64-apple-darwin` artifact, records the
exact transitive license inventory and SHA-256 manifest, and qualifies initial
install plus atomic same-version replacement without rewriting the external
version-1 profile. Publication, automatic update, wider platform support,
signing, notarization, and GUI distribution are explicitly deferred and are not
claims of the selected local product.

Run `sh qualification/agora-host/qualify.sh` to qualify one Pantheon process
owning Chronos, Vellum, and the real Agora local daemon while serving Agora's
public Unix client. The same process observes all three peers through the
headless aggregation API used by the standalone Pantheon CLI. The nested lock
closure also qualifies Agora's dedicated composition profile by returning the
same public `ScheduledLocalClientV1` from `NativeHost`, publishing once, and
observing the exact retained acceptance. Both scenarios use an explicitly
test-only Agora authority composition. Its command derives a scoped Cargo source patch because
Chronos standalone and Agora resolve the compatible checkpoint/commit subset of
`keystone-storage` from different local identities. It qualifies the all-three lifecycle graph,
but it is a test-only in-process topology, not a normal unpatched release build.
The selected production topology below supervises Agora as its sovereign
process and therefore does not merge its Cargo source graph with Chronos.

Run `sh tests/live/agora_hosted.sh` for the production-shaped process boundary.
Pantheon first hosts the real Aegis daemon and then Agora's exact standalone
private-local binary, observes both through their public clients, drains Agora
before Aegis, reopens both durable stores, and repeats the same path through the
interactive Agora component. A test-only process proxy then preserves a real
Agora cleanup result as residue, kills the exact child at the shutdown edge,
and withholds a terminal owner receipt through Pantheon's bounded deadline.
Pantheon reports refused or outcome-unknown rather than aggregate success,
still drains Aegis, waits for a safe replacement endpoint, and proves Agora's
explicit prior/next incarnation recovery retains the room. The profile carries only Agora's
executable, opaque config path, public socket, and explicit composition
transport; all authority, residue validation, and bootstrap policy remain
Agora-owned.

Run `sh tests/live/agora_mutation.sh` for selected Agora mutation parity. The
gate drives the same owner guard through Pantheon against the real daemon,
proving pre-connection confirmation refusal, stale-revision refusal and the
exact durable room-creation receipt. Agora's standalone real-daemon gate also
discards one accepted reply and proves the retained client request is replayed
unchanged; broader conversation mutations remain unclaimed.

Run `sh tests/live/aegis_attach.sh` to qualify the independently packaged
read-only Aegis peer through the real durable daemon in both Pantheon headless
and interactive attachment, including bounded grant/decision page navigation.
The gate exercises `authority.observe.v1` plus the peer-owned
`authority.revoke-epoch.v1`, `authority.attenuate-authorize-once.v1`, and
`aegis.admin-session.v1` contracts.
Pantheon gains no ambient authority: separately invoked commands require the
exact current epoch and literal confirmation, attempt each mutation once, and
never retry an uncertain result. The bounded authority journey keeps its parent
and child handles inside one authenticated connection and returns only redacted
authorization facts. The interactive workbench now composes that same owner as
a private 1–16-scope issue/attenuate/authorize session: exact generated
confirmations, explicit arm/confirm, at most eight retained grants, no retry
after dispatch, and only operation IDs plus redacted facts on screen or in
instrumentation.
The managed-session command keeps named grants inside Aegis's shared
bounded owner, supports issue/attenuate/authorize/list/close over NDJSON, and is
identical to the standalone shell below the socket/stdio composition edge.
Pantheon does not parse requests, retain handles, or redefine confirmations.
The gate also runs two separate Pantheon command processes against the selected
long-running owner daemon: the first abandons a named grant on disconnect, and
the second observes an empty local name set before deliberately rebuilding under
a different operation ID.

Run `sh tests/live/aegis_hosted.sh` to qualify Pantheon-owned lifecycle for
Aegis's standalone daemon. The profile supplies explicit create/open mode,
daemon path, durable directory, socket, authority namespace, OS peer identity,
principal, session namespace and role. Pantheon waits for the owner socket,
observes through the same public client in headless and interactive modes,
performs a real parent issue, child attenuation, and authorization inside one
instrumented private managed session, requests signal drain, verifies clean
socket removal and reopens the same authority. It links no Aegis
engine/service/storage crate and treats none of those deployment facts as a
grant.

Run `sh tests/live/tellus_attach.sh` to qualify the independently packaged
Tellus peer through Dendrite's typed authenticated observation endpoint in
both Pantheon modes. This uses `tellus.observe.v1`, not Dendrite's agent-facing
MCP boundary.

Run `sh tests/live/moira_attach.sh` to qualify Moira's content-free census in
headless mode and the same reusable component's bounded selected-saga evidence
and operator receipts interactively. The gate uses only `SagaReadOnlyClient`;
it grants Pantheon no reconciliation authority.

Run `sh tests/live/moira_hosted.sh` to qualify the sovereign Moira saga owner.
The journey seeds one durable saga through Moira's public client, drains the
standalone owner, then reopens it under Pantheon in headless and interactive
modes. Both paths observe the retained census and end with a clean supervised
receipt and removed socket; a final headless reopen proves persistence.

Run `sh tests/live/moira_reconciliation.sh` to qualify the selected production
Aegis profile and shared reconciliation guard. Standalone and Pantheon prepare
and confirm in opposite directions, both recover attributed receipts, an
unselected saga is refused without revision change, and Pantheon hosts and
cleanly drains the same durable owner before receipt recovery after reopen.

Run `sh tests/live/daimon_attach.sh` to qualify Daimon's redacted presence plus
one exact configured `agents.session.inspect.v1` projection in both Pantheon
modes. The agent projection double-reads the stable session around a bounded
contiguous event page and exposes phase, profile digest, capabilities and event
kinds without cataloguing sessions or acquiring open, prompt, permission,
advance, cancel, close, resume or reattach authority.

Run `sh tests/live/daimon_hosted.sh` to qualify the production
`daimon-local-agent` under Pantheon ownership. The gate issues a real
session-bound Aegis grant, validates one exact Chronos attempt, seeds a durable
resumable session through Daimon's public client, drains the standalone owner,
then reopens the same checkpoint through Pantheon in headless and interactive
modes. Both paths inspect the recovered `ReattachRequired` session through the
ordinary client and end with a clean Daimon shutdown receipt and removed owner
socket. Provider launch policy and credentials remain in the opaque owner
configuration; the qualification provider is never dispatched.

Run `sh tests/live/daimon_advance.sh` to qualify the separate
`agents.session.advance.v1` operator path. Standalone Daimon and Pantheon reuse
the same Daimon-owned exact-state guard: it observes the configured session on
the mutation connection, requires the exact active turn, revision, idempotency
key and confirmation, and attempts one bounded provider slice. An uncertain
delivery is never retried; retained session inspection is the reconciliation
path. The reusable Daimon presentation remains read-only.

Run `sh tests/live/daimon_cancel.sh` to qualify
`agents.session.cancel.v1`. Standalone Daimon and Pantheon reuse the same
owner guard and exact confirmation across turn-active and permission-waiting
states. Cancellation is attempted once with a stable idempotency key; an
uncertain delivery is never retried and must be reconciled by inspection.

Run `sh tests/live/daimon_close.sh` to qualify `agents.session.close.v1`.
Both shells reuse the owner guard that accepts only an exact Ready revision
with no active turn, so active work must be cancelled first. Close is attempted
once and an uncertain delivery remains an inspection/reconciliation problem.

Run `sh tests/live/daimon_batch.sh` to qualify the interactive bounded workset
against a real serial Daimon Unix owner. Pantheon observes three exact sessions
over one short-lived refresh connection, sparsely selects all three, then opens
one fresh connection and operation identity per guarded close. The gate proves
three distinct owner-returned Closed projections, three mutation receipts, and
a post-receipt refresh of the complete selected workset;
the unit-level uncertainty gate separately proves stop-on-unknown and NotSent
preservation. Connection instrumentation also holds a 16-session refresh to one
simultaneous connection and verifies release before subsequent work.

Run `sh tests/live/daimon_recovery_batch.sh` to qualify partial-run recovery.
The real owner accepts the first close, durably applies the second while its
reply becomes outcome-unknown, and proves the third child remains NotSent.
Pantheon retains the evidence sheet, blocks dismissal until `r` queues one
read-only refresh for every sent session, never retries the mutation, and keeps
the final trace uncertain even when current projections become available.
Mixed-state worksets are rejected before identity creation with bounded
position, phase, revision, and active-turn diagnostics rather than silently
dropping ineligible selections.
The session list and detail pane also show phase-derived owner-state candidates
beside every current projection while explicitly stating that authority is
still required; these hints improve discovery but cannot enable an action.

Run `sh tests/live/daimon_reattach.sh` to qualify
`agents.session.reattach.v1`. Both shells parse the same strict credential-free
execution document and reuse the owner guard that accepts only an exact
resume-capable `ReattachRequired` revision with no active turn. Confirmation
binds the complete fresh execution envelope; the single attempt neither
replays history nor submits a prompt, and uncertain delivery is not retried.

Run `sh tests/live/daimon_reattach_batch.sh` to qualify the interactive
session-set workflow. Pantheon requires every selected session to remain at an
exact resume-capable `ReattachRequired` revision, creates a fresh task,
execution, and idempotency identity for each child from the credential-free
policy, and presents the complete guarded proposal before any mutation. The
real serial owner gate proves three independent Ready projections, retained
child receipts, stop-on-unknown semantics, post-receipt refresh, and a complete
content-free instrumentation trace.

Run `sh tests/live/daimon_prompt.sh` to qualify
`agents.session.prompt.v1`. Standalone Daimon and Pantheon parse the same
strict credential-free request document and invoke the same owner guard. Prompt
bytes are read from a bounded file rather than argv; exact confirmation binds
their digest and the canonical digest of the entire execution envelope. The
guard fences the ready revision on the mutation connection, dispatches once,
and leaves transport uncertainty for retained-session reconciliation.

Run `sh tests/live/daimon_prompt_batch.sh` to qualify the actual interactive
prompt workflow. A versioned workspace profile supplies only the reusable,
credential-free principal/profile, context, tool, and budget policy. Pantheon
composes multiline text, reviews the selected 1–16 ready sessions, creates a
fresh turn, task, execution, and idempotency identity for every child, and then
hands the exact requests to Daimon's guarded batch component. The real serial
owner gate proves three independent TurnActive projections and a final
post-receipt session-set refresh; prompt content never enters instrumentation.

Run `sh tests/live/daimon_permission.sh` to qualify
`agents.session.permission.v1`. Both shells reuse the owner parser and guard to
select one cursor-identified challenge, exact request and option. Confirmation
binds digests of the complete provider challenge and separately authorized
execution envelope; changed provider text, turn, options or cursor refuse before
mutation, and uncertain decision delivery is never retried.

Run `sh tests/live/daimon_permission_batch.sh` to qualify the interactive
challenge-native workset. Pantheon requires every selected session to be at an
exact `AwaitingPermission` revision whose latest retained event is the active
turn's challenge. The review sheet presents each challenge's escaped capability,
description, and owner-published options, defaults independently to
`RejectOnce`, and permits a different explicit choice for every session. The
real serial owner gate proves three cursor-bound decisions, independent fresh
execution envelopes, post-receipt projection refresh, content-free telemetry,
and no latency-budget violation.

Run `sh tests/live/hermes_attach.sh` to qualify configured stream health and one
configured consumer's content-free cursor/backlog through Hermes's read-only
facade. Exact opaque selectors are profile inputs and are never catalogued or
echoed. The same gate separately qualifies `ledger.events.disclose.v1` with an
exact stream/start/limit confirmation, a 16-event ceiling, and the negotiated
frame bound. Pantheon's interactive surface narrows that owner capability to
an explicit 1–16 event batch per confirmation and retains only a 256-byte
preview per event. Routine
observation remains content-free; Pantheon receives no
append or compaction authority. The gate also qualifies
`ledger.acknowledge.v1`: standalone and Pantheon reuse one peer-owned guard
that observes the expected cursor and attempts one exact-confirmed
acknowledgement on the same connection. Consumer authority remains outside the
reusable component. A hosted daemon may select the exact-stream operator profile
explicitly; Pantheon never infers it from the presentation stream.
The gate also qualifies `ledger.compact.v1` through the same pattern: exact
retained-frontier and published-edge observation precede one confirmed
compaction, with no retry after uncertain delivery. Operator authority remains
outside the reusable component. A hosted daemon may receive one explicit
exact-stream operator profile; Pantheon owns only its lifecycle and passes that
selection to Hermes unchanged.

Run `sh tests/live/hermes_hosted.sh` to qualify Pantheon-owned lifecycle for
Hermes's standalone selected daemon. The profile supplies explicit
create/recover mode, daemon path, durable directory, socket, OS peer identity,
subject and observer/operator selection. Operator selection additionally names
one exact stream and must match any presentation stream. Pantheon waits for the
owner socket, observes through the same public client used for attachment in
headless and interactive modes, requests signal drain, verifies clean shutdown,
and reopens the same ledger after each path. The selected operator case invokes
the same exact-confirmed acknowledgement and compaction guards while Pantheon
hosts the owner, then proves the cursor and retention edge survive reopen. It
links no Hermes engine/service or Keystone crate and does not acquire or infer
ledger authority.

Run `sh tests/live/operational_attach.sh` to qualify the real Moira, Daimon,
Hermes, and Aegis daemons plus Dendrite's typed Tellus service in one
Pantheon headless invocation and one Ratatui session. The live Overview must
render all six peer evidence channels—including Daimon's distinct presence and
session/event channels—with explicit current or degraded evidence state. This
coexistence gate exercises only peer-owned observation capabilities; it does
not claim native hosting or mutation authority for those peers.

Run `sh tests/live/reconfiguration_restart.sh` to qualify the interactive
configuration journey without peer fixtures: exact shell/profile diff,
arm/confirm, terminal-safe process replacement, and inspection of the
replacement session's selected surfaces.

Run `sh tests/live/tui_breadth.sh` to launch the real binary under tmux and
exercise all eleven first-class surfaces, mnemonic and numeric direct
navigation, sequential navigation, Configuration disclosure, host refresh,
palette, help, activity, and safe exit.

Run `sh tests/live/reconfiguration_hosted_restart.sh` for the full lifecycle
case. Pantheon hosts one real Chronos topology, preflights a different hosted
workstream as a full local-topology restart, requires a clean old-owner drain
before process replacement, observes the replacement owner as ready, and then
independently reopens and cleanly drains the old topology to prove its locks and
endpoints were released.
