# Workbench TUI

`workbench-tui` is an unpublished reusable terminal presentation host for
independent workspace peers. The sibling `workbench-tui-cli` package owns the
standalone executable, profile parsing, terminal lifecycle, and process exit;
the root package owns navigation, layout, focus, and global input routing. Each
attached peer renders only inside its assigned rectangle and retains its own
input and service authority.

The binary attaches a Chronos Hub and one Vellum Unix service socket. Add
`--chronos-workstream NAME DIRECTORY` for each extra Chronos service. With
Chronos focused, **w** switches workstreams and **/** searches observed tasks. It does not orchestrate agents, merge engine state or infer
task completion from edits. The host preserves Vellum's dirty/indeterminate exit
laws and never retries an uncertain effect. Chronos writes remain disabled
unless explicitly enabled; Vellum authority remains entirely service-owned.

The versioned workspace shell also names Overview, Agora, Moira, Daimon,
Grounding, Aegis and Hermes surfaces. Overview and the Chronos/Vellum Workspace
are live in the standalone binary. A workspace profile may attach Agora through
the selected private Unix client by supplying its socket and participant claim;
the daemon authenticates the connected OS peer and remains the authority owner.
A trusted host can instead supply an independently constructed `agora-tui`
component through `workbench_tui_cli::run_hosted_terminal_with_agora`. Moira,
Daimon, Hermes, Aegis, and Grounding profiles attach their peer-owned read-only
components. Grounding uses Dendrite's typed presentation endpoint rather than
MCP tool envelopes or text. `[` and `]` switch
surfaces; Tab switches focus only inside Workspace. See
[the gap ledger](WORKSPACE_GAPS.md) and [delivery roadmap](ROADMAP.md).

```sh
cargo run -p workbench-tui-cli --bin workbench-tui --locked --offline -- interactive \
  /private/tmp/chronos-service-directory \
  --vellum-service /private/tmp/vellum.sock
```

To load a bounded workspace profile instead, copy
`config/workspace.example.json`, adjust its relative service paths, and run:

```sh
cargo run -p workbench-tui-cli --bin workbench-tui --locked --offline -- interactive \
  --workspace-profile config/workspace.json
```

Validate profile structure and path resolution without contacting a service:

```sh
cargo run -p workbench-tui-cli --bin workbench-tui --locked --offline -- interactive \
  --workspace-profile config/workspace.json --check-profile
```

Profiles contain connection configuration, presentation selection and explicit
cross-service scope mappings. They cannot enable Chronos writes or carry grant
handles. Continue to pass `--chronos-allow-writes` at launch when intended.
Agora, Chronos, Vellum, Moira, Daimon, Hermes and Aegis profiles may select `"deployment":"hosted"`. Workbench then
owns their real local service lifecycles while the same peer clients and TUI
components are used. Agora, Daimon, Hermes and Aegis remain sovereign supervised owner
processes rather than linked semantic engines. Hosted Vellum requires a bounded non-empty `roots` list;
attached profiles must not include roots. Edit and save authority remain
launch-only through `--vellum-allow-edit` and `--vellum-allow-save`.
Each configured peer may also carry a bounded `requires` list. Workbench checks
those exact identifiers against the peer-owned TUI manifest for the selected
interactive or headless mode before connecting. A match describes composition
support only; the service still owns authorization and may refuse the request.
The Agora participant value is an identity claim checked against the daemon's
authenticated peer context; it is neither a credential nor authority.
Unknown fields, versions, duplicate surfaces, control characters, excessive
workstreams/bindings and files larger than 64 KiB are refused.

A Daimon profile may pair `agent_socket` with one exact 32-hex-digit
`agent_session`. Workbench then reuses Daimon's read-only client to present the
stable session view and bounded event page in both headless and interactive
modes. The two fields are all-or-nothing; they confer no open, prompt,
permission, reattach, cancel, or close authority.

A hosted Daimon profile requires that exact agent selection plus an owner
`host` containing only the `daimon-local-agent` executable and its opaque
configuration path. The owner configuration retains provider commands,
inherited credential names, Aegis/Chronos endpoints, checkpoint directory and
limits. Workbench invokes Daimon's `--supervised` lifecycle, waits for the
configured public agent socket, uses the same public client as attach mode, and
requests a bounded owner-defined drain on exit. A presence socket remains
optional because the production agent application does not claim to serve the
separate legacy presence protocol.

A hosted Moira profile supplies only the `moira-saga-local` executable and its
opaque owner configuration. That Moira document retains the database, kernel
peer mapping, all bounds and the selected operator-authority policy. Workbench
invokes the exact supervised lifecycle, waits for the private saga socket and
attaches through `moira-client::unix::SagaUnixPort`; it cannot turn ordinary
read/mutate access into reconciliation authority.

The separate `moira-reconciliation-view`, `moira-reconciliation-confirm` and
`moira-reconciliation-recover` commands reuse Moira's owner-side guard. They
accept only an exact server-prepared challenge, submit confirmation once without
retrying an unknown outcome, and resolve uncertainty only from the exact durable
operator receipt. Workbench never parses the Aegis portion of the owner config.

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
profile or lifecycle receipt. Tab changes focus and Ctrl-Q requests
whole-workbench exit.
Use the explicit `headless` command for a bounded, versioned JSON observation of every configured
Agora, Chronos, Vellum, Moira, Daimon, Hermes, Aegis, and Grounding peer. Each row names its peer-owned capability and
truthful status. Healthy rows remain visible when another peer is unavailable,
but the command refuses aggregate success. `--headless` and `--snapshot` remain
compatibility aliases for invocations without an explicit command.

A Vellum-owned integration host can also call `workbench_tui_cli::run_hosted_terminal`
with a Chronos `Hub` and a factory for the Vellum service and handoff handler.
Editor commands and handoff requests share one service owner. The host resolves
the document association and authorizes capture, publication, and review.
The standalone socket CLI starts with handoff controls disabled.

The Workbench executable can natively host Chronos and Vellum together in
interactive or headless mode. Services bind before clients are constructed;
startup refusal rolls back earlier owners, and shutdown drains in reverse order
with explicit clean, refused, or outcome-unknown receipts. It can also supervise
Agora's exact `agora-private-local` process from an opaque owner config, wait for
its public socket, select the routed public client explicitly, request Agora's
stdin drain, and validate the bounded cleanup receipt. Workbench never parses
the config's Aegis, grant, clock, bootstrap, or durable-storage facts.

The workspace-wide standalone/Workbench pattern, composition taxonomy, current
parity matrix and Axon/Engram prior-art routing live in
[`docs/composition/`](docs/composition/README.md). Peer roadmaps continue to own
their behavior; this repository owns product composition and its evidence. The
closed machine-readable parity matrix is [`composition-status.json`](composition-status.json).

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
  grounding-tui/
  keystone/
  moira/
  moira-tui/
  vellum/
  vellum-tui/
  workbench-tui/
```

Run `cargo test --workspace --all-targets --locked --offline`. Nothing is published to
crates.io and no remote, release, signing or activation action is performed.
The machine source receipt records the clean committed local closure. Remote
CI and release qualification remain deferred until those exact inputs are
remotely reachable.

The source receipt now pins the committed Aegis, Agora, Chronos, Daimon, Grounding, Hermes, Moira, and Vellum TUI
boundaries used by this composition. Their reusable default presentations keep
standalone CLI/application concerns outside Workbench's dependency graph.
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

Run `sh tests/live/dual_service.sh` for local composition qualification.
It first builds and attaches temporary Agora, Chronos, and Vellum services,
reopens a saved profile with relative service paths, and uses an isolated tmux
session to check workstream switching, offline recovery, focus routing, all
three peer panes, and exit. It then restarts Chronos and Vellum under Workbench
ownership while attaching Agora, qualifies the mixed topology's headless
snapshot and interactive panes, and proves reverse shutdown removes every
Workbench-owned data and control endpoint.

Run `sh tests/live/chronos_mutation.sh` for the selected Chronos mutation
profile through the real Workbench-hosted peer component. The isolated journey
creates a task, enables scheduling, changes capacity and not-before, pauses the
task and scheduler, performs receipted cancellation, drains Workbench, then
reopens the state with an independent `chronos-service` and verifies the exact
durable result. This qualifies that profile locally; it does not invent mutation
capability IDs or grant wider authority.

Run `sh qualification/agora-host/qualify.sh` to qualify one Workbench process
owning Chronos, Vellum, and the real Agora local daemon while serving Agora's
public Unix client. The same process observes all three peers through the
headless aggregation API used by the standalone Workbench CLI. The nested lock
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
Workbench first hosts the real Aegis daemon and then Agora's exact standalone
private-local binary, observes both through their public clients, drains Agora
before Aegis, reopens both durable stores, and repeats the same path through the
interactive Agora component. A test-only process proxy then preserves a real
Agora cleanup result as residue, kills the exact child at the shutdown edge,
and withholds a terminal owner receipt through Workbench's bounded deadline.
Workbench reports refused or outcome-unknown rather than aggregate success,
still drains Aegis, waits for a safe replacement endpoint, and proves Agora's
explicit prior/next incarnation recovery retains the room. The profile carries only Agora's
executable, opaque config path, public socket, and explicit composition
transport; all authority, residue validation, and bootstrap policy remain
Agora-owned.

Run `sh tests/live/aegis_attach.sh` to qualify the independently packaged
read-only Aegis peer through the real durable daemon in both Workbench headless
and interactive attachment, including bounded grant/decision page navigation.
The gate exercises `authority.observe.v1` plus the peer-owned
`authority.revoke-epoch.v1`, `authority.attenuate-authorize-once.v1`, and
`aegis.admin-session.v1` contracts.
Workbench gains no ambient authority: separately invoked commands require the
exact current epoch and literal confirmation, attempt each mutation once, and
never retry an uncertain result. The bounded authority journey keeps its parent
and child handles inside one authenticated connection and returns only redacted
authorization facts. Reusable management is not exposed as one-shot commands.
The managed-session command keeps named grants inside Aegis's shared
bounded owner, supports issue/attenuate/authorize/list/close over NDJSON, and is
identical to the standalone shell below the socket/stdio composition edge.
Workbench does not parse requests, retain handles, or redefine confirmations.
The gate also runs two separate Workbench command processes against the selected
long-running owner daemon: the first abandons a named grant on disconnect, and
the second observes an empty local name set before deliberately rebuilding under
a different operation ID.

Run `sh tests/live/aegis_hosted.sh` to qualify Workbench-owned lifecycle for
Aegis's standalone daemon. The profile supplies explicit create/open mode,
daemon path, durable directory, socket, authority namespace, OS peer identity,
principal, session namespace and role. Workbench waits for the owner socket,
observes through the same public client in headless and interactive modes,
requests signal drain, verifies clean socket removal and reopens the same
authority. It links no Aegis engine/service/storage crate and treats none of
those deployment facts as a grant.

Run `sh tests/live/grounding_attach.sh` to qualify the independently packaged
Grounding peer through Dendrite's typed authenticated observation endpoint in
both Workbench modes. This uses `grounding.observe.v1`, not Dendrite's agent-facing
MCP boundary.

Run `sh tests/live/moira_attach.sh` to qualify Moira's content-free census in
headless mode and the same reusable component's bounded selected-saga evidence
and operator receipts interactively. The gate uses only `SagaReadOnlyClient`;
it grants Workbench no reconciliation authority.

Run `sh tests/live/moira_hosted.sh` to qualify the sovereign Moira saga owner.
The journey seeds one durable saga through Moira's public client, drains the
standalone owner, then reopens it under Workbench in headless and interactive
modes. Both paths observe the retained census and end with a clean supervised
receipt and removed socket; a final headless reopen proves persistence.

Run `sh tests/live/moira_reconciliation.sh` to qualify the selected production
Aegis profile and shared reconciliation guard. Standalone and Workbench prepare
and confirm in opposite directions, both recover attributed receipts, an
unselected saga is refused without revision change, and Workbench hosts and
cleanly drains the same durable owner before receipt recovery after reopen.

Run `sh tests/live/daimon_attach.sh` to qualify Daimon's redacted presence plus
one exact configured `agents.session.inspect.v1` projection in both Workbench
modes. The agent projection double-reads the stable session around a bounded
contiguous event page and exposes phase, profile digest, capabilities and event
kinds without cataloguing sessions or acquiring open, prompt, permission,
advance, cancel, close, resume or reattach authority.

Run `sh tests/live/daimon_hosted.sh` to qualify the production
`daimon-local-agent` under Workbench ownership. The gate issues a real
session-bound Aegis grant, validates one exact Chronos attempt, seeds a durable
resumable session through Daimon's public client, drains the standalone owner,
then reopens the same checkpoint through Workbench in headless and interactive
modes. Both paths inspect the recovered `ReattachRequired` session through the
ordinary client and end with a clean Daimon shutdown receipt and removed owner
socket. Provider launch policy and credentials remain in the opaque owner
configuration; the qualification provider is never dispatched.

Run `sh tests/live/daimon_advance.sh` to qualify the separate
`agents.session.advance.v1` operator path. Standalone Daimon and Workbench reuse
the same Daimon-owned exact-state guard: it observes the configured session on
the mutation connection, requires the exact active turn, revision, idempotency
key and confirmation, and attempts one bounded provider slice. An uncertain
delivery is never retried; retained session inspection is the reconciliation
path. The reusable Daimon presentation remains read-only.

Run `sh tests/live/daimon_cancel.sh` to qualify
`agents.session.cancel.v1`. Standalone Daimon and Workbench reuse the same
owner guard and exact confirmation across turn-active and permission-waiting
states. Cancellation is attempted once with a stable idempotency key; an
uncertain delivery is never retried and must be reconciled by inspection.

Run `sh tests/live/daimon_close.sh` to qualify `agents.session.close.v1`.
Both shells reuse the owner guard that accepts only an exact Ready revision
with no active turn, so active work must be cancelled first. Close is attempted
once and an uncertain delivery remains an inspection/reconciliation problem.

Run `sh tests/live/daimon_reattach.sh` to qualify
`agents.session.reattach.v1`. Both shells parse the same strict credential-free
execution document and reuse the owner guard that accepts only an exact
resume-capable `ReattachRequired` revision with no active turn. Confirmation
binds the complete fresh execution envelope; the single attempt neither
replays history nor submits a prompt, and uncertain delivery is not retried.

Run `sh tests/live/daimon_prompt.sh` to qualify
`agents.session.prompt.v1`. Standalone Daimon and Workbench parse the same
strict credential-free request document and invoke the same owner guard. Prompt
bytes are read from a bounded file rather than argv; exact confirmation binds
their digest and the canonical digest of the entire execution envelope. The
guard fences the ready revision on the mutation connection, dispatches once,
and leaves transport uncertainty for retained-session reconciliation.

Run `sh tests/live/daimon_permission.sh` to qualify
`agents.session.permission.v1`. Both shells reuse the owner parser and guard to
select one cursor-identified challenge, exact request and option. Confirmation
binds digests of the complete provider challenge and separately authorized
execution envelope; changed provider text, turn, options or cursor refuse before
mutation, and uncertain decision delivery is never retried.

Run `sh tests/live/hermes_attach.sh` to qualify configured stream health and one
configured consumer's content-free cursor/backlog through Hermes's read-only
facade. Exact opaque selectors are profile inputs and are never catalogued or
echoed. The same gate separately qualifies `ledger.events.disclose.v1` with an
exact stream/start/limit confirmation, a 16-event ceiling, and the negotiated
frame bound. Routine observation remains content-free; Workbench receives no
append or compaction authority. The gate also qualifies
`ledger.acknowledge.v1`: standalone and Workbench reuse one peer-owned guard
that observes the expected cursor and attempts one exact-confirmed
acknowledgement on the same connection. Consumer authority remains outside the
reusable component. A hosted daemon may select the exact-stream operator profile
explicitly; Workbench never infers it from the presentation stream.
The gate also qualifies `ledger.compact.v1` through the same pattern: exact
retained-frontier and published-edge observation precede one confirmed
compaction, with no retry after uncertain delivery. Operator authority remains
outside the reusable component. A hosted daemon may receive one explicit
exact-stream operator profile; Workbench owns only its lifecycle and passes that
selection to Hermes unchanged.

Run `sh tests/live/hermes_hosted.sh` to qualify Workbench-owned lifecycle for
Hermes's standalone selected daemon. The profile supplies explicit
create/recover mode, daemon path, durable directory, socket, OS peer identity,
subject and observer/operator selection. Operator selection additionally names
one exact stream and must match any presentation stream. Workbench waits for the
owner socket, observes through the same public client used for attachment in
headless and interactive modes, requests signal drain, verifies clean shutdown,
and reopens the same ledger after each path. The selected operator case invokes
the same exact-confirmed acknowledgement and compaction guards while Workbench
hosts the owner, then proves the cursor and retention edge survive reopen. It
links no Hermes engine/service or Keystone crate and does not acquire or infer
ledger authority.

Run `sh tests/live/operational_attach.sh` to qualify the real Moira, Daimon,
Hermes, and Aegis daemons plus Dendrite's typed Grounding service in one
Workbench headless invocation. This coexistence gate exercises only the five
peer-owned observation capabilities; it does not claim native hosting or
mutation authority for those peers.
